#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
BACKUP_FILE="${1:?usage: restore_db.sh <backup.dump>}"

if [[ ! -f "$BACKUP_FILE" ]]; then
  echo "backup file not found: $BACKUP_FILE" >&2
  exit 1
fi

if [[ -f "$BACKUP_FILE.sha256" ]]; then
  sha256sum -c "$BACKUP_FILE.sha256"
fi

echo "RESTORE WILL REPLACE DATA IN DATABASE_URL."
echo "Set CONFIRM_RESTORE=YES to continue."
[[ "${CONFIRM_RESTORE:-}" == "YES" ]] || exit 2

pg_restore   --clean   --if-exists   --no-owner   --no-privileges   --dbname="$DATABASE_URL"   "$BACKUP_FILE"

echo "restore=ok"
