#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${FORMAT_BASE_SHA:-}" ]]; then
  mapfile -t changed_rust_files < <(
    git diff --name-only --diff-filter=ACMR "${FORMAT_BASE_SHA}" HEAD -- '*.rs'
  )
  for file in "${changed_rust_files[@]}"; do
    rustfmt --edition 2021 --check "${file}"
  done
  echo "INCREMENTAL RUST FORMAT CHECK PASSED"
else
  cargo fmt --all -- --check
  echo "FULL RUST FORMAT CHECK PASSED"
fi
