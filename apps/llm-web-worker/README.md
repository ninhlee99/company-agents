# LLM Web Session Worker

The worker consumes short-lived LLM jobs from `llm-web-relay` and executes them using either a user-controlled browser adapter or an official subscription CLI.

## Adapter modes

### ChatGPT subscription

Use the official Codex CLI authenticated with the user's ChatGPT account:

```
WEB_SESSION_BACKEND=chatgpt-web
WEB_SESSION_ADAPTER_KIND=codex-chatgpt
CODEX_BIN=codex
```

The worker invokes read-only `codex exec --json` and accepts only the final JSON object.

### Claude Pro/Max subscription

Use Claude Code authenticated with the user's Claude account:

```
WEB_SESSION_BACKEND=claude-web
WEB_SESSION_ADAPTER_KIND=claude-code
CLAUDE_BIN=claude
```

The worker invokes non-interactive `claude -p` in plan mode with one turn and requires a JSON object result.

### Gemini / generic consumer web

Use the browser-side adapter:

```
WEB_SESSION_BACKEND=gemini-web
WEB_SESSION_ADAPTER_KIND=browser-command
WEB_SESSION_ADAPTER_COMMAND=/absolute/path/to/browser-adapter
```

The browser adapter owns the persistent consumer-web session. Credentials and cookies never cross the relay boundary.

## Common environment

```
LLM_RELAY_URL=http://127.0.0.1:9010
LLM_RELAY_WORKER_TOKEN=<long-random-worker-token>
WEB_SESSION_ADAPTER_TIMEOUT_SECONDS=45
WEB_SESSION_WORKDIR=/tmp/company-agents-llm
```

The worker enforces:
- backend allow-listing (`gemini-web`, `chatgpt-web`, `claude-web`);
- JSON-only jobs;
- `allow_tools=false`;
- bounded prompt/output sizes;
- lease-token completion;
- bounded subprocess execution with kill-on-drop;
- no CAPTCHA/MFA/access-control bypass.

Keep the worker on the machine that owns the user subscription/browser session. The worker does not proxy browser cookies to Company OS.

## Reliability boundary

The architecture is:

```
Company OS
  -> WebRelayModel
  -> PostgreSQL durable relay queue
  -> llm-web-worker
  -> Codex / Claude Code / browser adapter
```

The deterministic Company OS Governor remains authoritative after the LLM response returns.

## Credential isolation

The worker clears the adapter subprocess environment before launch.

By default only basic process variables such as PATH, HOME, USER, TMPDIR, TERM, and XDG_CONFIG_HOME are passed through. Add a variable explicitly with:

    WEB_SESSION_ENV_ALLOWLIST=CODEX_ACCESS_TOKEN,CLAUDE_CODE_OAUTH_TOKEN

Do not place unrelated company secrets in this allowlist. Prefer the CLI's own local login storage under the isolated worker user's home directory when supported.
