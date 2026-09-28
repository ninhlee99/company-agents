# Consumer Web LLM Bridge Protocol

This protocol is the explicit boundary between Company OS and a user's already-authenticated browser session.

## Contract

1. Company OS submits `POST /v1/generate` to `llm-web-relay`.
2. Relay stores the job in PostgreSQL and returns only when a worker completes it.
3. A browser bridge calls `POST /v1/jobs/claim` with `backend=gemini-web`, `chatgpt-web`, or `claude-web`.
4. The bridge opens the corresponding provider web application in a user-owned browser session.
5. The bridge submits only the supplied system/user prompt, waits for the final answer, validates that it is one JSON object, and calls `/complete`.
6. The relay returns the validated JSON to Company OS.

## Security boundary

The repository never receives browser passwords or provider session cookies.
The browser bridge should use a browser profile that the operator has already authenticated and should not export its cookies.
The bridge must not grant tools or execute Company OS commands. `allow_tools` is always false for relay jobs.
The bridge should run on the operator's machine or another explicitly trusted host and use the worker token only for relay access.

## Backends

- `gemini-web`: Gemini consumer web application.
- `chatgpt-web`: ChatGPT consumer web application.
- `claude-web`: Claude consumer web application.

Web UI layouts are not stable APIs. Provider-specific DOM/browser automation belongs behind this contract so the Rust Company OS does not depend on undocumented selectors or session internals.

## Job example

    POST /v1/generate
    Authorization: Bearer <relay-api-token>
    Content-Type: application/json

    {
      "protocol_version": 1,
      "backend": "gemini-web",
      "model": "web-session",
      "system": "Return exactly one JSON object. Do not execute tools.",
      "user": "...",
      "response_format": "json_object",
      "allow_tools": false
    }

The bridge then claims the job, receives a lease token, and completes it with:

    {
      "lease_token": "<uuid>",
      "output": "{\"action\":\"ProduceReport\"}"
    }

Stale leases are rejected. Expired jobs cannot be completed. A job can be retried up to the configured maximum attempt count.

## Why this is separate from official APIs

Official APIs are the primary production integration path. The relay is an optional compatibility layer for a user who explicitly wants to use an already-authenticated web session.
This separation prevents a web session from silently becoming the Company's authority boundary.