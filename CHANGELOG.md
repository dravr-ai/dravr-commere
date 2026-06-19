# Changelog

## [0.1.7] — 2026-06-19

### Changed

- deps: migrate `dravr-commere-mcp` and `dravr-commere-server` to dravr-tronc
  0.5.3 (dual-era MCP engine); state is `Arc<S>` directly (tronc no longer wraps
  it in a `RwLock`). The core `dravr-commere` crate is unchanged.

## [0.1.6] — 2026-05-10

### Fixed

- fix(pg): stringify data + actions JSON; TEXT columns silently dropped Value bind The PG notifications.data and notifications.actions columns are TEXT (per pierre-platform migration 20260311000007 + the 20260401000001 actions backfill); binding serde_json::Value directly let sqlx map to JSONB at the protocol layer, the JSONB→TEXT cast failed, and the bind degraded to NULL with no error — Recovery / Coach notification deep links and action buttons never persisted in production. Mirror SQLite's pattern: stringify before bind, parse on read.

### Other

- chore(deps): bump rustls-webpki 0.103.11→0.103.13, rand 0.8.5→0.8.6, rand 0.9.2→0.9.3 Closes Dependabot #6 (HIGH rustls-webpki CRL DoS), #5 #4 (LOW rand bias), #3 #2 (LOW rustls-webpki name-constraints). All transitive — Cargo.lock only.



## [0.1.4] — 2026-04-10

### Other

- build: reduce tokio feature footprint to minimal set



## [0.1.3] — 2026-04-01



## [0.1.2] — 2026-03-31



## [0.1.1] — 2026-03-26

### Other

- deps: bump dravr-tronc to 0.2 with error notification support



## [0.0.2] — 2026-03-23

### Other

- refactor: adopt dravr-tronc shared MCP infrastructure



## [0.0.1] — 2026-03-23

### Added

- feat: add API request/response DTOs and collapse_notifications Complete the notification domain model with feed response types, preference DTOs, scheduled notification items, and notification collapsing logic.


