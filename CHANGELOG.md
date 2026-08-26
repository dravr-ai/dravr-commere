# Changelog

## [0.3.2] — 2026-08-26

### Fixed

- fix(postgres): stored preference JSON and quiet hours are read as text on
  PostgreSQL. `sub_preferences` and the quiet-hour boundaries are TEXT on both
  engines; the PostgreSQL mapper decoded them as native `serde_json::Value`
  and `NaiveTime` and hid the type failure with `unwrap_or(None)`, so every
  preference read back without its per-type toggles and quiet hours. The
  upsert now binds them as text too, and the read accepts the `HH:MM:SS`
  spelling the earlier `time`-typed writes left behind.

## [0.3.1] — 2026-08-26

### Fixed

- fix(postgres): notification ids are bound and read as text on PostgreSQL.
  Every notification table stores its id as TEXT holding a UUID string (the
  schema dravr-platform ships for both engines, and what the SQLite
  repository already does), but the PostgreSQL repository bound a native
  `Uuid` for every id and decoded the id column as a native UUID, so no
  notification row could be written or read on PostgreSQL. Surfaced by
  dravr-platform's PostgreSQL-lane notification test.

## [0.3.0] — 2026-08-26

### Removed

- refactor(actions)!: the `AcceptDecline` notification action type is deleted —
  `NotificationActionType::AcceptDecline`, its `"accept_decline"` wire string,
  and the iOS `categoryIdentifier` the dispatcher derived from it. The
  friend-request trigger deleted in 0.2.0 was its only producer, so no
  notification has carried the action since; dravr-platform drops the matching
  shared-types union member in the same change. `OpenScreen` and `QuickReply`
  are unchanged.

## [0.2.0] — 2026-08-26

### Removed

- refactor(social)!: the `Social` notification category is deleted —
  `NotificationCategory::Social` and its `"social"` stored string, the four
  social triggers (`trigger_friend_request_received`,
  `trigger_friend_request_accepted`, `trigger_activity_kudos`,
  `trigger_insight_shared`), the `kudos_received` / `friend_request`
  feed-collapse rules, and `constants::FAN_OUT_PAGE_SIZE`. dravr-platform,
  the only consumer, retired the Insights and Friends surfaces by deletion
  (Chat-First Cutover), so nothing raises a social notification any more;
  `from_str_opt("social")` now returns `None`, and the platform's migration
  deletes the stored `social` rows before this version reads them.

### Fixed

- fix(hooks): the SessionStart hook arms `.build/hooks` instead of a missing
  `.githooks`, so Claude sessions no longer run with the git hooks disabled.


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


