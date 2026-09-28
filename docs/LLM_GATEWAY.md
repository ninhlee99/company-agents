# LLM Gateway

Company OS now talks to a provider-neutral `Model` interface. Agent code does not know which model is serving the request.

## Provider modes

`LLM_PROVIDER=ollama` keeps the current local-first deployment.

`LLM_PROVIDER=gemini` uses the official Gemini Interactions API. Google documents Interactions as the recommended primitive for agentic workflows, with REST support and API-key authentication.

`LLM_PROVIDER=openai` uses the OpenAI Responses API. This is the developer API and is separate from the ChatGPT consumer web application.

`LLM_PROVIDER=anthropic` uses the Anthropic Messages API. This is the developer API and is separate from Claude consumer web sessions.

`LLM_PROVIDER=web-relay` uses the Company OS Web Relay protocol. The relay is intentionally a separate process because consumer web sessions are not a supported public inference API. The relay may be backed by a user-authenticated browser/session, but Company OS never receives browser cookies, passwords, or account tokens. The relay is responsible for its own session handling and website compatibility.

## Failover

Set `LLM_FALLBACKS` to a comma-separated ordered list:

```
LLM_PROVIDER=web-relay
LLM_FALLBACKS=gemini,anthropic,ollama
```

The gateway tries providers in order. A provider returning malformed JSON or transport failure is treated as unavailable. No provider is allowed to execute tools; it only returns a proposal JSON object. The existing Governor, execution engine, economic limits, permissions, and persistence remain authoritative.

## Web Relay protocol

Request:

```json
{
  "protocol_version": 1,
  "backend": "gemini-web",
  "model": "web-session",
  "system": "...",
  "user": "...",
  "response_format": "json_object",
  "allow_tools": false
}
```

Response:

```json
{
  "output": "{\"action\":\"ProduceReport\",\"confidence\":0.8}"
}
```

Required properties:
- HTTPS for non-loopback relays.
- Bearer token authentication.
- No redirects.
- Maximum request size 512 KiB.
- Maximum response size 1 MiB.
- The relay must not return browser secrets.
- `allow_tools` must remain false for the current Company Agent proposal path.

## Recommended deployment pattern

For a web-subscription-backed setup, run a local or private relay on the same machine/network as the authenticated browser profile. The relay translates the protocol above into a browser-side operation and returns only the model text.

This design deliberately avoids undocumented provider-internal HTTP endpoints and avoids putting consumer session cookies into Company OS.

## What "web" means here

Gemini API, OpenAI API, and Anthropic API are official programmatic interfaces. Google currently recommends the Gemini Interactions API for new agentic applications. OpenAI exposes its developer API through the OpenAI Platform. Anthropic distinguishes its developer platform/API from its consumer Claude applications.

The web-relay mode is therefore an adapter boundary, not a claim that the consumer web products themselves expose a stable public REST API.

## Configuration examples

Official API backend:

```
LLM_PROVIDER=gemini
LLM_FALLBACKS=anthropic,ollama
```

Consumer-web relay:

```
LLM_PROVIDER=web-relay
LLM_WEB_RELAY_URL=https://relay.example.internal/v1/generate
LLM_WEB_RELAY_TOKEN=<secret>
LLM_WEB_RELAY_BACKEND=claude-web
LLM_WEB_RELAY_MODEL=claude-web-session
LLM_FALLBACKS=gemini,ollama
```

The relay itself is intentionally not part of the core autonomy boundary until it has its own browser-session isolation, authentication, rate limiting, audit, and crash-recovery tests.
