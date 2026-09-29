# LLM Gateway

Company OS agents depend only on the Rust `Model` trait. The selected backend is an implementation detail of the gateway.

## Programmatic providers

- `ollama` / `local`: local OpenAI-compatible endpoint.
- `gemini`: Google Gemini Interactions API.
- `openai` / `chatgpt`: OpenAI Responses API.
- `anthropic` / `claude`: Anthropic Messages API.

These are programmatic developer APIs. They are the stable production path for server-side inference.

## Consumer web providers

The following names are supported:

- `gemini-web`
- `chatgpt-web`
- `claude-web`

These do **not** call undocumented consumer endpoints from Company OS. They call the internal `llm-web-relay` service. The relay queues the job in PostgreSQL; a separate browser bridge running on the user's machine claims the job and interacts with the already-authenticated browser session.

The browser profile remains local to the browser machine. Company OS never receives provider cookies, passwords, or session storage.

## Provider selection

Primary provider:

```
LLM_PROVIDER=ollama
```

Web provider:

```
LLM_PROVIDER=gemini-web
LLM_WEB_RELAY_URL=http://127.0.0.1:9010/v1/generate
LLM_WEB_RELAY_TOKEN=<company-to-relay-secret>
LLM_WEB_RELAY_BACKEND=gemini-web
LLM_WEB_RELAY_MODEL=web-session
```

For the dedicated aliases, the provider name controls the web backend:

```
LLM_PROVIDER=gemini-web
LLM_PROVIDER=chatgpt-web
LLM_PROVIDER=claude-web
```

Ordered fallback:

```
LLM_PROVIDER=web-relay
LLM_FALLBACKS=gemini,anthropic,ollama
```

The gateway tries providers in order. A malformed response, timeout, or provider transport error causes the next provider to be tried.

## Security boundary

The gateway never lets an LLM directly execute Company OS tools. The model returns proposal JSON only.

The deterministic layers remain authoritative:

`Agent policy → Proposal validation → Governor → Execution engine → PostgreSQL transaction → audit/outbox`

The web relay adds another deterministic boundary:

`Company OS → API bearer token → durable relay job → worker bearer token → browser session`

The relay rejects:

- unknown web backends
- non-v1 jobs
- tool execution requests
- non-JSON-object responses
- oversize prompts
- oversize outputs
- stale or invalid lease tokens

## Browser bridge

Start the relay:

```
docker compose --profile web-relay up --build
```

Then on the browser machine:

```
cd apps/llm-web-bridge
python -m venv .venv
. .venv/bin/activate
pip install -r requirements.txt
playwright install chromium
export LLM_RELAY_URL=http://127.0.0.1:9010
export LLM_RELAY_WORKER_TOKEN=<worker-secret>
export LLM_WEB_BRIDGE_BACKEND=gemini-web
export LLM_WEB_BRIDGE_HEADLESS=false
python bridge.py
```

Login to the desired consumer web application interactively in the persistent Chromium profile. The bridge subsequently reuses that local profile.

Switching providers:

```
LLM_WEB_BRIDGE_BACKEND=chatgpt-web python bridge.py
LLM_WEB_BRIDGE_BACKEND=claude-web python bridge.py
```

## Reliability

Relay jobs have:

- PostgreSQL durability
- idempotency keys
- attempt counters
- expiry
- worker leases
- lease tokens
- stale-worker rejection
- explicit failure/requeue behavior

An old browser worker cannot complete a job after another worker has reclaimed it.

## Important limitation

Consumer web UIs are not stable backend APIs. The selectors in `apps/llm-web-bridge/bridge.py` are intentionally isolated from Rust so UI changes can be fixed without changing Company OS. Production environments should run selector contract tests against the current web UI and keep official provider APIs as fallback.


## Subscription-backed configuration

### ChatGPT subscription via Codex

Set:

```
LLM_PROVIDER=chatgpt-subscription
LLM_WEB_RELAY_BACKEND=chatgpt-web
WEB_SESSION_ADAPTER_KIND=codex-chatgpt
WEB_SESSION_BACKEND=chatgpt-web
```

Authenticate the Codex CLI on the worker machine with the user's ChatGPT account, then run:

```bash
WEB_SESSION_BACKEND=chatgpt-web \
WEB_SESSION_ADAPTER_KIND=codex-chatgpt \
cargo run -p llm-web-worker
```

The worker runs one read-only `codex exec --json` job at a time.

### Claude Pro/Max via Claude Code

Set:

```
LLM_PROVIDER=claude-subscription
LLM_WEB_RELAY_BACKEND=claude-web
WEB_SESSION_ADAPTER_KIND=claude-code
WEB_SESSION_BACKEND=claude-web
```

Authenticate Claude Code on the worker machine with the user's Claude account, then run:

```bash
WEB_SESSION_BACKEND=claude-web \
WEB_SESSION_ADAPTER_KIND=claude-code \
cargo run -p llm-web-worker
```

The worker uses one non-interactive `claude -p` job with a single turn and plan permissions.

### Gemini consumer web

Gemini consumer-web access remains a browser-session adapter:

```
LLM_PROVIDER=gemini-web
LLM_WEB_RELAY_BACKEND=gemini-web
WEB_SESSION_ADAPTER_KIND=browser-command
WEB_SESSION_BACKEND=gemini-web
WEB_SESSION_ADAPTER_COMMAND=/absolute/path/to/browser-adapter
```

There is deliberately no "Gemini subscription API" shortcut in the Company OS. Consumer web sessions stay inside the user-controlled browser profile. Start the browser bridge with:

```bash
cd apps/llm-web-bridge
LLM_WEB_BRIDGE_BACKEND=gemini-web python bridge.py
```

