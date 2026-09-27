#!/usr/bin/env bash
set -euo pipefail

while IFS= read -r file; do
  if grep -nE '^DO \$$' "$file" >/dev/null 2>&1; then
    echo "MIGRATION LINT FAILED: malformed DO delimiter in $file" >&2
    exit 1
  fi
  if grep -nE '^DO \$[^$]' "$file" >/dev/null 2>&1; then
    echo "MIGRATION LINT FAILED: malformed DO block opener in $file" >&2
    exit 1
  fi
done < <(find infra/db/migrations -type f -name '*.sql' | sort)

while IFS= read -r file; do
  bash -n "$file"
done < <(find scripts -type f -name '*.sh' | sort)

echo "INFRA LINT PASSED"