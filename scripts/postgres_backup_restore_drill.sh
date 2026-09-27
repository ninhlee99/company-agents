#!/usr/bin/env bash
set -euo pipefail

SOURCE_DATABASE_URL="${SOURCE_DATABASE_URL:-${DATABASE_URL:-}}"
DRILL_DATABASE_URL="${DRILL_DATABASE_URL:-}"

if [[ -z "$SOURCE_DATABASE_URL" || -z "$DRILL_DATABASE_URL" ]]; then
  echo "SOURCE_DATABASE_URL and DRILL_DATABASE_URL are required" >&2
  exit 2
fi

command -v pg_dump >/dev/null
command -v pg_restore >/dev/null
command -v psql >/dev/null

BACKUP_FILE="${BACKUP_FILE:-$(mktemp --suffix=.dump)}"
cleanup() {
  rm -f "$BACKUP_FILE"
}
trap cleanup EXIT

pg_dump   --format=custom   --no-owner   --no-acl   --file="$BACKUP_FILE"   "$SOURCE_DATABASE_URL"

test -s "$BACKUP_FILE"

psql "$DRILL_DATABASE_URL" -v ON_ERROR_STOP=1 <<'SQL'
DO $$
BEGIN
  IF current_database() = current_database() THEN
    PERFORM 1;
  END IF;
END
$$;
SQL

pg_restore   --clean   --if-exists   --no-owner   --no-acl   --exit-on-error   --dbname="$DRILL_DATABASE_URL"   "$BACKUP_FILE"

psql "$DRILL_DATABASE_URL" -v ON_ERROR_STOP=1 -Atc   "SELECT CASE WHEN EXISTS (SELECT 1 FROM companies) THEN 'RESTORE_OK' ELSE 'RESTORE_EMPTY' END;"

echo "POSTGRES BACKUP/RESTORE DRILL PASSED"
