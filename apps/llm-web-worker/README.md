# LLM Web Session Worker

This process consumes short-lived jobs from the Company OS web relay and passes them to a user-controlled browser adapter.

The Company OS never receives browser cookies or consumer-web credentials. The worker only receives:
- backend: `gemini-web`, `chatgpt-web`, or `claude-web`
- model
- system prompt
- user prompt
- strict JSON-output requirement

The browser adapter is an external executable selected with `WEB_SESSION_ADAPTER_COMMAND`. It receives one JSON object on stdin and must print one JSON object on stdout.

Use one worker per browser profile when isolation is required. Keep the browser session on the user's own machine. Do not automate CAPTCHA/MFA bypasses or defeat provider rate/access controls.

Example environment:

```
LLM_RELAY_URL=http://127.0.0.1:9010
LLM_RELAY_WORKER_TOKEN=<long-random-token>
WEB_SESSION_ADAPTER_COMMAND=/absolute/path/to/browser-adapter
WEB_SESSION_BACKEND=gemini-web
WEB_SESSION_ADAPTER_TIMEOUT_SECONDS=45
```

The worker enforces bounded prompt/output sizes, backend allow-listing, lease tokens, timeouts, and JSON-object-only outputs.
