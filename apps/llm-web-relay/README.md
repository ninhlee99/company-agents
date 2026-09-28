# LLM Web Relay

This service is the Company OS boundary for consumer-web LLM sessions.

It does not log in to Gemini, ChatGPT, or Claude, does not store browser cookies, and does not call undocumented provider-internal APIs.

## Flow

1. Company OS POSTs a JSON-only generation job to /v1/generate.
2. The relay persists the job in PostgreSQL.
3. A browser worker authenticates with the worker token and claims one job at a time.
4. The worker submits the prompt to an already authenticated browser session.
5. The worker returns one JSON object to /v1/jobs/{id}/complete.
6. The relay releases the waiting Company OS request.
7. Lease tokens prevent an old browser worker from completing a job after another worker has reclaimed it.

## Endpoints

Company OS token:
- POST /v1/generate
- GET /metrics

Browser-worker token:
- POST /v1/jobs/claim
- POST /v1/jobs/{job_id}/complete
- POST /v1/jobs/{job_id}/fail

Unauthenticated:
- GET /healthz

## Security boundary

The API token and worker token must be different.

A relay job is valid only when:
- protocol_version = 1
- backend is gemini-web, chatgpt-web, or claude-web
- response_format = json_object
- allow_tools = false
- model is at most 128 bytes
- prompt payload is at most 512 KiB
- output is a JSON object of at most 1 MiB

Each worker claim receives a unique lease_token. Completion and failure are accepted only with the current lease token.

## Browser session

The browser side must keep the user session local. A persistent browser profile can be used so that the user logs in interactively once and the worker reuses that profile later.

The relay never receives browser cookies, passwords, session storage, or provider account tokens.

Provider UI selectors are deliberately kept in the browser worker instead of Company OS because consumer web interfaces can change independently of the Company OS backend. The worker should expose selector configuration so changes do not require rebuilding the core system.

## Docker

Run the relay only when it is intentionally configured:

    docker compose --profile web-relay up --build

Set:
- LLM_RELAY_API_TOKEN
- LLM_RELAY_WORKER_TOKEN

For Company OS in the same compose network:

    LLM_PROVIDER=web-relay
    LLM_FALLBACKS=gemini,anthropic,ollama
    LLM_WEB_RELAY_URL=http://llm-web-relay:9010/v1/generate
    LLM_WEB_RELAY_ALLOW_HTTP_HOSTS=llm-web-relay

For a remote deployment, put the relay behind HTTPS and omit the HTTP allowlist.

## Important

The consumer web products are not treated as stable public inference APIs. The official programmatic backends remain the preferred integration path. The relay is an adapter boundary for a user-authenticated browser session.

