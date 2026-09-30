#!/usr/bin/env bash
set -euo pipefail

bash scripts/check_rust_format.sh
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "ALL RUST VERIFY GATES PASSED"
