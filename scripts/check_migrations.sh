#!/usr/bin/env bash
set -euo pipefail

for file in infra/db/migrations/*.sql; do
  if grep -nE '^DO \$$|^DO \$[^$]|^\$;$' "$file" >/dev/null 2>&1; then
    echo "MIGRATION LINT FAILED: malformed dollar quote in $file" >&2
    exit 1
  fi
  opens=$(grep -cE '^DO \$\$' "$file" || true)
  closes=$(grep -cE '^\$\$;$' "$file" || true)
  if [[ "$opens" -ne "$closes" ]]; then
    echo "MIGRATION LINT FAILED: unbalanced dollar quote in $file" >&2
    exit 1
  fi
done

echo "MIGRATION CHECK PASSED"
