import asyncio, json, os
from pathlib import Path
import httpx
from playwright.async_api import async_playwright

S = {
 "chatgpt-web": {
  "url":"https://chatgpt.com/",
  "input":["#prompt-textarea","textarea[data-testid='textbox']","div[contenteditable='true']"],
  "send":["button[data-testid='send-button']","button[aria-label*='Send']"],
  "reply":["[data-message-author-role='assistant']"]},
 "gemini-web": {
  "url":"https://gemini.google.com/app",
  "input":["rich-textarea","div[contenteditable='true']","textarea"],
  "send":["button[aria-label*='Send']","button[aria-label*='send']"],
  "reply":["model-response",".model-response-text"]},
 "claude-web": {
  "url":"https://claude.ai/",
  "input":["div[contenteditable='true']","textarea"],
  "send":["button[aria-label*='Send']","button[aria-label*='send']"],
  "reply":["div[class*='standard-markdown']","[data-testid='chat-message']"]},
}

async def first_visible(page, selectors, timeout_ms=15000):
    end = asyncio.get_running_loop().time() + timeout_ms / 1000
    while asyncio.get_running_loop().time() < end:
        for selector in selectors:
            try:
                loc = page.locator(selector)
                if await loc.count() and await loc.last.is_visible():
                    return loc.last
            except Exception:
                pass
        await asyncio.sleep(0.2)
    raise TimeoutError("no visible browser selector matched")

def make_prompt(job):
    return (
        "COMPANY AGENT SYSTEM INSTRUCTIONS\n"
        "--- BEGIN SYSTEM ---\n" + job["system"] +
        "\n--- END SYSTEM ---\n\n"
        "USER DATA\n--- BEGIN USER DATA ---\n" + job["user"] +
        "\n--- END USER DATA ---\n\n"
        "Return exactly one JSON object. Do not call tools."
    )

def normalize(text):
    text = text.strip()
    value = json.loads(text)
    if not isinstance(value, dict):
        raise ValueError("output must be a JSON object")
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if len(encoded.encode()) > 1024 * 1024:
        raise ValueError("output exceeds 1 MiB")
    return encoded

async def run_job(page, job):
    spec = S[job["backend"]]
    if job["protocol_version"] != 1 or job["allow_tools"] or job["response_format"] != "json_object":
        raise ValueError("unsafe or unsupported relay job")

    await page.goto(spec["url"], wait_until="domcontentloaded")
    before = {s: await page.locator(s).count() for s in spec["reply"]}
    editor = await first_visible(page, spec["input"], min(120000, max(5000, job["expires_in_ms"])))
    await editor.fill(make_prompt(job))
    await (await first_visible(page, spec["send"])).click()

    deadline = asyncio.get_running_loop().time() + min(120000, max(5000, job["expires_in_ms"])) / 1000
    while asyncio.get_running_loop().time() < deadline:
        for selector in spec["reply"]:
            try:
                loc = page.locator(selector)
                if await loc.count() > before.get(selector, 0):
                    text = (await loc.last.inner_text()).strip()
                    if text:
                        return normalize(text)
            except Exception:
                pass
        await asyncio.sleep(0.5)
    raise TimeoutError("web model response timeout")

async def main():
    relay = os.getenv("LLM_RELAY_URL", "http://127.0.0.1:9010").rstrip("/")
    token = os.getenv("LLM_RELAY_WORKER_TOKEN", "").strip()
    profile = Path(os.getenv("LLM_WEB_BROWSER_PROFILE", str(Path.home()/".company-agents-web-profile"))).expanduser()
    backend = os.getenv("LLM_WEB_BRIDGE_BACKEND", "").strip() or None
    if len(token) < 16:
        raise RuntimeError("LLM_RELAY_WORKER_TOKEN must be at least 16 characters")
    profile.mkdir(parents=True, exist_ok=True)

    async with httpx.AsyncClient(base_url=relay, headers={"Authorization": f"Bearer {token}"}, timeout=30) as client:
        async with async_playwright() as p:
            ctx = await p.chromium.launch_persistent_context(
                user_data_dir=str(profile),
                headless=os.getenv("LLM_WEB_BRIDGE_HEADLESS", "false").lower() in {"1","true","yes","on"},
                accept_downloads=False)
            page = ctx.pages[0] if ctx.pages else await ctx.new_page()
            try:
                while True:
                    response = await client.post("/v1/jobs/claim", json={"backend": backend})
                    response.raise_for_status()
                    job = response.json()
                    if not job:
                        await asyncio.sleep(float(os.getenv("LLM_WEB_BRIDGE_POLL_SECONDS", "1")))
                        continue
                    try:
                        output = await run_job(page, job)
                        done = await client.post(
                            f"/v1/jobs/{job['job_id']}/complete",
                            json={"lease_token":job["lease_token"],"output":output})
                        done.raise_for_status()
                    except Exception as exc:
                        failed = await client.post(
                            f"/v1/jobs/{job['job_id']}/fail",
                            json={"lease_token":job["lease_token"],"error":str(exc)[:4096]})
                        failed.raise_for_status()
            finally:
                await ctx.close()

if __name__ == "__main__":
    asyncio.run(main())
