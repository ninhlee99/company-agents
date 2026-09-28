#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
: "${PGHOST:?PGHOST is required}"
: "${PGPORT:?PGPORT is required}"
: "${PGUSER:?PGUSER is required}"
: "${PGPASSWORD:?PGPASSWORD is required}"
DB_NAME="${PGDATABASE:-company_agents}"
WORKDIR="${TMPDIR:-/tmp}/company-agents-recovery-$$"
mkdir -p "$WORKDIR"
trap 'rm -rf "$WORKDIR"' EXIT

psql -v ON_ERROR_STOP=1 "$DATABASE_URL" <<'SQL'
CREATE EXTENSION IF NOT EXISTS pgcrypto;
DROP TABLE IF EXISTS recovery_drill_marker;
CREATE TABLE recovery_drill_marker (id integer PRIMARY KEY, payload text NOT NULL);
INSERT INTO recovery_drill_marker VALUES (1, 'recovery-ok');
SQL

pg_dump --no-owner --no-privileges --format=custom "$DATABASE_URL" > "$WORKDIR/db.dump"
dropdb --if-exists --host "$PGHOST" --port "$PGPORT" --username "$PGUSER" "$DB_NAME"
createdb --host "$PGHOST" --port "$PGPORT" --username "$PGUSER" "$DB_NAME"
pg_restore --no-owner --no-privileges --dbname="$DATABASE_URL" "$WORKDIR/db.dump"
restored="$(psql -At -v ON_ERROR_STOP=1 "$DATABASE_URL" -c "SELECT payload FROM recovery_drill_marker WHERE id = 1;")"
test "$restored" = "recovery-ok"
echo "RECOVERY DRILL PASSED"