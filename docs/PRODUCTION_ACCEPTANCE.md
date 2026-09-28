# Production Acceptance Standard

The Company OS is considered production-ready only when every gate below is green.

## Automated gates

- Rust formatting, workspace tests and Clippy with `-D warnings`.
- Migration delimiter lint.
- Tracked-secret hygiene.
- Docker production image builds.
- PostgreSQL schema initialization.
- PostgreSQL backup/restore drill.

These gates run in GitHub Actions on every push/PR, with the recovery drill also scheduled weekly.

## Runtime gates

- `/healthz` is liveness only.
- `/readyz` verifies the database-backed control plane.
- `/metrics` exposes Prometheus-compatible runtime counters.
- Durable scheduler jobs use database leases and replay-safe run tokens.
- Durable outbox events use leases, bounded retries and optional HMAC signatures.
- Publishing requires an explicit approval token and execution lease.
- Affiliate revenue is not treated as verified until provider reconciliation authorizes it.
- Agent memory and rate limits are persisted transactionally.

## Secrets

Production secrets must be supplied through an external secret manager or mounted secret files. Do not commit `.env` files, browser cookies, API tokens or private keys.

Rotate a secret by:
1. Create the new secret in the external secret manager.
2. Deploy it as a new secret version.
3. Set `SECRET_VERSION` to the new version identifier.
4. Restart/redeploy workers.
5. Verify `/readyz`, worker logs and downstream authentication.
6. Revoke the previous secret only after the new version is confirmed healthy.

## Recovery

At least weekly, run the PostgreSQL restore drill against an isolated database. A successful drill must prove that:
- a backup can be created;
- the backup can be restored with `ON_ERROR_STOP`;
- the restored schema is queryable;
- at least one company row survives restoration.

A production release must not skip recovery validation merely because application tests pass.

## External integrations

Real platform credentials, advertiser contracts, publisher accounts and payment rails are environment-specific acceptance inputs. The repository provides bounded adapters and approval contracts, but cannot manufacture or validate third-party credentials without the operator's accounts.
