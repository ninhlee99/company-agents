# Backup and Recovery

The Company OS treats the PostgreSQL database as the source of truth for company state, ledger, decision journal, agent memory, affiliate attribution and operational jobs.

## Backup

Use `scripts/postgres_backup_restore_drill.sh` with:
- `SOURCE_DATABASE_URL`: production/source database.
- `DRILL_DATABASE_URL`: isolated disposable restore target.
- `BACKUP_FILE`: optional custom dump path.

The drill uses PostgreSQL custom-format dumps and restores into a separate database. It never writes to the source database.

## Recovery requirements

A release is not production-ready until:
- a backup can be created successfully;
- a restore completes with `--exit-on-error`;
- restored data contains at least one company row;
- the restored database can run the economic and Company Store integration tests;
- recovery time and recovery point objectives are documented for the deployment.

Agent execution remains fail-closed when durable state is unavailable.
