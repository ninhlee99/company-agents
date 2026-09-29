# Consumer Web LLM Bridge Protocol

The bridge is deliberately separated from Company OS.

Flow:

`Company OS -> LLM Web Relay -> Browser Bridge -> Gemini / ChatGPT / Claude Web -> Relay -> Company OS`

The relay accepts only:

- backend: `gemini-web`, `chatgpt-web`, `claude-web`
- JSON-object responses
- `allow_tools=false`
- bounded request and response sizes
- worker leases and idempotency keys

The browser bridge keeps its persistent browser profile on the operator machine. Browser cookies and passwords are never sent to Company OS or stored in PostgreSQL.

The bridge owns only prompt submission and response extraction. It cannot execute Company tools, change Agent permissions, or bypass Governor decisions.

Consumer web UIs are not stable APIs, so DOM selectors are isolated in `apps/llm-web-bridge/bridge.py`. A selector change should not require a Rust control-plane change.

For production-critical workloads, official Gemini/OpenAI/Anthropic APIs remain the preferred programmatic path.
