#!/usr/bin/env bash
set -euo pipefail

patterns=(
  'AKIA[0-9A-Z]{16}'
  'gh[pousr]_[A-Za-z0-9_]{20,}'
  'sk-[A-Za-z0-9]{20,}'
  'xox[baprs]-[A-Za-z0-9-]{20,}'
  '-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----'
)

for pattern in "${patterns[@]}"; do
  if git grep -nI -E "$pattern" --       ':!*.md' ':!.env.example' ':!docs/**' ':!scripts/check_secret_hygiene.sh' >/tmp/secret-hits 2>/dev/null; then
    echo "SECRET HYGIENE FAILED: credential-like material detected" >&2
    cat /tmp/secret-hits >&2
    exit 1
  fi
done

if git ls-files --error-unmatch .env >/dev/null 2>&1; then
  echo "SECRET HYGIENE FAILED: .env is tracked" >&2
  exit 1
fi

echo "SECRET HYGIENE PASSED"
