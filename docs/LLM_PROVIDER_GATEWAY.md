# LLM Provider Gateway

## Purpose
Company OS talks to a single `Model` contract. Provider selection, fallback and transport details stay outside Agent logic.

Supported backends:

| LLM_PROVIDER | Backend | Credential | Notes |
|---|---|---|---|
| `ollama` | Local Ollama | none | Default offline/local path |
| `gemini` | Google Gemini API | `GEMINI_API_KEY` | Provider API path |
| `openai` / `chatgpt` | OpenAI API | `OPENAI_API_KEY` | Provider API path, not the consumer ChatGPT web app |
| `anthropic` / `claude` | Anthropic API | `ANTHROPIC_API_KEY` | Claude Messages API |
| `web-relay` | Consumer-web relay | relay token | Generic browser bridge |
| `gemini-web` | Gemini consumer-web relay | relay token | Browser bridge selects Gemini |
| `chatgpt-web` | ChatGPT consumer-web relay | relay token | Browser bridge selects ChatGPT |
| `claude-web` | Claude consumer-web relay | relay token | Browser bridge selects Claude |
| `mock` | Deterministic mock | none | Tests and offline development |

## Important distinction

A consumer ChatGPT subscription is not the same thing as the OpenAI developer API. OpenAI documents separate billing systems, so the Company OS keeps `OPENAI_API_KEY` separate from a ChatGPT subscription.

Gemini exposes an official API and an OpenAI-compatible endpoint. The gateway supports the Gemini API path directly and keeps the OpenAI-compatible transport available for compatibility.

Anthropic exposes the Claude Messages API and publishes a model lifecycle/deprecation schedule, so the gateway keeps the Claude model configurable.

## Shadow model routing

The gateway supports a deterministic routing recommendation layer in `MODEL_ROUTER_MODE=shadow` (the safe default). Each model call is classified as **Fast**, **Standard**, or **Deep**, the local logical-CPU count is mapped to a **Small**, **Medium**, or **Large** hardware tier, and a configured provider is recommended for telemetry.

The recommendation is advisory only: the existing primary-provider/fallback order remains the provider actually used. Deep work on small machines prefers a configured remote API provider; fast work prefers a configured local/mock provider. Large prompts are promoted to the Deep class. Active provider switching is intentionally not enabled until benchmark and acceptance evidence exist.

## Recommended production mode

Use official APIs as primary providers and the consumer-web relay only when a browser bridge is intentionally deployed.

`LLM_PROVIDER=openai`
`LLM_FALLBACKS=gemini,anthropic,ollama`

The exact model stays configurable because provider catalogs change. The current example uses `gpt-5.6-luna` for cost-sensitive high-volume work.

## Consumer-web relay

`apps/llm-web-relay` is a durable queue/lease service. It does not store browser passwords or session cookies.

Flow:

`Company OS -> /v1/generate -> PostgreSQL job -> browser bridge -> /jobs/{id}/complete`

The browser-side component is responsible for using an already-authenticated browser session and returning one JSON object. The server receives only bounded prompt data and the final JSON result.

The relay rejects tool execution (`allow_tools=false`) and bounds request/response sizes. API and worker tokens are separate.

This keeps the Company OS independent from undocumented provider web endpoints.

## Configuration examples

### Gemini API

    LLM_PROVIDER=gemini
    GEMINI_API_KEY=...
    GEMINI_MODEL=gemini-3.8-flash

### OpenAI API

    LLM_PROVIDER=openai
    OPENAI_API_KEY=...
    OPENAI_MODEL=gpt-5.6-luna

### Claude API

    LLM_PROVIDER=anthropic
    ANTHROPIC_API_KEY=...
    ANTHROPIC_MODEL=claude-opus-4-8

### Web relay

    LLM_PROVIDER=gemini-web
    LLM_WEB_RELAY_URL=http://127.0.0.1:9010/v1/generate
    LLM_WEB_RELAY_TOKEN=...
    LLM_WEB_RELAY_MODEL=web-session

Start the relay profile:

    docker compose --profile web-relay up

## Safety guarantees

The gateway never grants an Agent extra permission because a model/provider requested it. Provider output is untrusted JSON. Governance, action capability, cost ceilings, execution limits, durable state and material side effects remain deterministic.

Provider failure, malformed JSON, timeout, rate-limit denial or relay expiry fail closed into escalation instead of executing an external/material action.