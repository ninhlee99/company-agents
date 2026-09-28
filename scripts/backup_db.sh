#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
OUTPUT_DIR="${1:-backups}"
mkdir -p "$OUTPUT_DIR"

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
archive="$OUTPUT_DIR/company_agents_$timestamp.dump"
sha="$archive.sha256"

pg_dump "$DATABASE_URL" --format=custom --no-owner --no-privileges --file="$archive"
sha256sum "$archive" > "$sha"

echo "backup=$archive"
echo "checksum=$sha"
