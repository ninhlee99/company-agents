#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
: "${DRILL_DATABASE_URL:?DRILL_DATABASE_URL is required}"

BACKUP_DIR="${1:-backups/drill}"
mkdir -p "$BACKUP_DIR"

echo "[1/4] create backup"
bash scripts/backup_db.sh "$BACKUP_DIR"

LATEST="$(ls -1t "$BACKUP_DIR"/*.dump | head -n1)"
echo "[2/4] verify checksum"
sha256sum -c "$LATEST.sha256"

echo "[3/4] restore into drill database"
CONFIRM_RESTORE=YES DATABASE_URL="$DRILL_DATABASE_URL" bash scripts/restore_db.sh "$LATEST"

echo "[4/4] smoke test database"
psql "$DRILL_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM companies;"
psql "$DRILL_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM ledger_transactions;"

echo "recovery_drill=ok"
