#!/usr/bin/env bash
set -euo pipefail

for file in infra/db/migrations/*.sql; do
  python3 - "$file" <<'PY'
import re
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

def dollar_quotes_are_balanced(source: str) -> tuple[bool, str]:
    i = 0
    state = "normal"
    tag = None
    while i < len(source):
        if state == "normal":
            if source.startswith("--", i):
                state = "line_comment"
                i += 2
                continue
            if source.startswith("/*", i):
                state = "block_comment"
                i += 2
                continue
            if source[i] == "'":
                state = "single_quote"
                i += 1
                continue
            if source[i] == '"':
                state = "double_quote"
                i += 1
                continue
            match = re.match(r"\$([A-Za-z_][A-Za-z0-9_]*)?\$", source[i:])
            if match:
                tag = match.group(0)
                state = "dollar_quote"
                i += len(tag)
                continue
            i += 1
        elif state == "line_comment":
            if source[i] == "\n":
                state = "normal"
            i += 1
        elif state == "block_comment":
            if source.startswith("*/", i):
                state = "normal"
                i += 2
            else:
                i += 1
        elif state == "single_quote":
            if source.startswith("''", i):
                i += 2
            elif source[i] == "'":
                state = "normal"
                i += 1
            else:
                i += 1
        elif state == "double_quote":
            if source.startswith('""', i):
                i += 2
            elif source[i] == '"':
                state = "normal"
                i += 1
            else:
                i += 1
        elif state == "dollar_quote":
            end = source.find(tag, i)
            if end < 0:
                return False, tag
            i = end + len(tag)
            tag = None
            state = "normal"
    if state == "dollar_quote":
        return False, tag or "$$"
    return True, ""

ok, unclosed = dollar_quotes_are_balanced(text)
if not ok:
    print(f"MIGRATION LINT FAILED: unbalanced dollar quote {unclosed} in {path}", file=sys.stderr)
    raise SystemExit(1)
PY
done

echo "MIGRATION CHECK PASSED"
