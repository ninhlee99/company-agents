# Recovery Drill

The Company OS treats PostgreSQL as the durable source of truth for company state, execution idempotency, the decision journal, outbox events, affiliate attribution and media jobs.

## Backup

Set `DATABASE_URL` and run:

```bash
bash scripts/backup_db.sh backups
```

The script creates a PostgreSQL custom-format dump plus a SHA-256 checksum.

## Restore

Restore is intentionally destructive and requires explicit confirmation:

```bash
CONFIRM_RESTORE=YES \
DATABASE_URL=postgresql://... \
bash scripts/restore_db.sh backups/company_agents_YYYYMMDDTHHMMSSZ.dump
```

The restore script verifies the checksum when the `.sha256` file is present.

## Full drill

Use an isolated disposable PostgreSQL database:

```bash
DRILL_DATABASE_URL=postgresql://... \
DATABASE_URL=postgresql://... \
bash scripts/recovery_drill.sh
```

The drill verifies that a backup can be restored and that core tables are readable afterward.

## Scheduler recovery

Scheduled jobs use:

- `FOR UPDATE SKIP LOCKED` to avoid duplicate claims;
- a lease (`locked_until`) so crashed workers become reclaimable;
- a persistent `run_token` so retrying the same job reuses the same cycle idempotency key;
- token rotation only after successful completion;
- failure release with a short retry delay.

## Recovery acceptance

A deployment is not considered recovery-ready until:

1. a production-like backup has been created successfully;
2. checksum verification succeeds;
3. the dump restores into an isolated database;
4. `companies`, `company_state_snapshots`, `ledger_transactions`, `agent_runs`, `decision_journal`, `outbox_events`, `agent_memory`, `affiliate_conversions` and `media_jobs` are readable;
5. a scheduler crash simulation demonstrates that a leased job can be reclaimed without double-applying the same idempotent cycle.

The drill is intentionally separate from the normal application startup path.
