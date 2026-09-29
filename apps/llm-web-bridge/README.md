# LLM Web Bridge

This is an optional browser-side worker for the Company OS LLM Web Relay.

It uses a persistent local Chromium profile. The user logs into Gemini, ChatGPT, or Claude interactively in that profile. The worker later reuses that session to execute queued JSON-only agent prompts.

## Run

    python -m venv .venv
    . .venv/bin/activate
    pip install -r requirements.txt
    playwright install chromium

Set:

    LLM_RELAY_URL=http://127.0.0.1:9010
    LLM_RELAY_WORKER_TOKEN=<worker-secret>
    LLM_WEB_BRIDGE_BACKEND=gemini-web
    LLM_WEB_BROWSER_PROFILE=~/.company-agents-web-profile
    LLM_WEB_BRIDGE_HEADLESS=false

Then start:

    python bridge.py

For ChatGPT web use:

    LLM_WEB_BRIDGE_BACKEND=chatgpt-web python bridge.py

For Claude web use:

    LLM_WEB_BRIDGE_BACKEND=claude-web python bridge.py

## Session isolation

The profile directory stays on the browser machine. The worker never sends cookies or credentials to the relay.

The worker allows only these fixed origins:
- https://gemini.google.com/app
- https://chatgpt.com/
- https://claude.ai/

A relay job cannot request arbitrary navigation or tool execution.

## Selector maintenance

Consumer web UIs can change their DOM without notice. Selectors are intentionally isolated in bridge.py so a UI change does not affect the Rust Company OS.

The current defaults are best-effort and must be validated against the current UI before production use.

## Production boundary

This worker is an experimental adapter for user-authenticated consumer web sessions. It is not an official Gemini, ChatGPT, or Claude API integration. Official developer APIs remain the stable programmatic path.

