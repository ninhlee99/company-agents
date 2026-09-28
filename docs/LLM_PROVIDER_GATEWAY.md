# LLM Provider Gateway

Company OS uses one internal `Model` contract and can route the same Agent workload to:

- Ollama local
- Gemini API
- OpenAI API
- Anthropic API
- optional consumer-web relay for Gemini, ChatGPT, or Claude

Provider selection:

```env
LLM_PROVIDER=openai
LLM_FALLBACKS=gemini,anthropic,ollama
LLM_STRICT_CONFIG=true
```

Consumer web subscriptions are not treated as developer APIs. Web-session mode is an explicit browser bridge: Company OS sends a bounded job to the relay; the browser-side bridge uses an already authenticated user profile and returns one JSON object.

The LLM never receives Company execution authority. Governor validation, tool permissions, spending limits, persistence, idempotency and external side-effect gates remain deterministic.

Official APIs should be preferred for stable production integration. The web relay exists for cases where the operator intentionally wants to reuse an authenticated consumer web session.

Model identifiers remain configurable because provider catalogs and lifecycle policies change over time.
