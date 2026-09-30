#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
: "${PGHOST:?PGHOST is required}"
: "${PGPORT:?PGPORT is required}"
: "${PGUSER:?PGUSER is required}"
: "${PGPASSWORD:?PGPASSWORD is required}"

WORKDIR="${TMPDIR:-/tmp}/company-agents-recovery-$$"
mkdir -p "$WORKDIR"
cleanup() {
  rm -rf "$WORKDIR"
  if [[ -n "${DRILL_DB_NAME:-}" ]]; then
    dropdb --if-exists --host "$PGHOST" --port "$PGPORT" --username "$PGUSER" "$DRILL_DB_NAME" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

# The source is read-only. The drill never drops or mutates it.
DRILL_DB_NAME="${PGDATABASE:-company_agents}_recovery_$"
SOURCE_COMPANIES_BEFORE="$(psql -At -v ON_ERROR_STOP=1 "$DATABASE_URL" -c "SELECT count(*) FROM companies;")"

pg_dump --no-owner --no-privileges --format=custom "$DATABASE_URL" > "$WORKDIR/db.dump"
createdb --host "$PGHOST" --port "$PGPORT" --username "$PGUSER" "$DRILL_DB_NAME"
pg_restore --no-owner --no-privileges --exit-on-error   --dbname="$DRILL_DB_NAME" "$WORKDIR/db.dump"

restored="$(PGDATABASE="$DRILL_DB_NAME" psql -At -v ON_ERROR_STOP=1 -c   "SELECT CASE WHEN EXISTS (SELECT 1 FROM companies) THEN 'RESTORE_OK' ELSE 'RESTORE_EMPTY' END;")"

test "$restored" = "RESTORE_OK"
SOURCE_COMPANIES_AFTER="$(psql -At -v ON_ERROR_STOP=1 "$DATABASE_URL" -c "SELECT count(*) FROM companies;")"
test "$SOURCE_COMPANIES_BEFORE" = "$SOURCE_COMPANIES_AFTER"
echo "RECOVERY DRILL PASSED"

