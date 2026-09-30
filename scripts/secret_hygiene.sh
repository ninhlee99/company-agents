#!/usr/bin/env bash
set -euo pipefail

tracked="$(git ls-files)"
for pattern in 'AKIA[0-9A-Z]{16}' 'ghp_[A-Za-z0-9]{20,}' 'github_pat_[A-Za-z0-9_]{20,}' 'sk-[A-Za-z0-9]{20,}' 'AIza[0-9A-Za-z_-]{20,}'; do
  if printf '%s\n' "$tracked" | xargs -r grep -nE "$pattern" -- 2>/dev/null; then
    echo "SECRET HYGIENE FAILED: tracked secret-like token detected" >&2
    exit 1
  fi
done

for forbidden in GEMINI_API_KEY AWIN_ACCESS_TOKEN AWIN_PRODUCT_FEED_API_KEY; do
  matches="$(printf '%s\n' "$tracked" | xargs -r grep -nE "(^|[^A-Z0-9_])${forbidden}=.{8,}" -- 2>/dev/null || true)"
  if [[ -n "$matches" ]]; then
    echo "SECRET HYGIENE FAILED: value assigned to $forbidden in tracked source" >&2
    exit 1
  fi
done

echo "SECRET HYGIENE PASSED"