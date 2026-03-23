# Commere — Push Notification Service

[![CI](https://github.com/dravr-ai/dravr-commere/actions/workflows/ci.yml/badge.svg)](https://github.com/dravr-ai/dravr-commere/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE.md)

Standalone multi-tenant push notification service with dispatch pipeline, preference-driven delivery, cron scheduling, and Expo Push integration. Dual SQLite/PostgreSQL backends with zero platform coupling in the core crate.

## Table of Contents

- [Quick Start](#quick-start)
- [Dispatch Pipeline](#dispatch-pipeline)
- [Notification Categories](#notification-categories)
- [Triggers](#triggers)
- [REST API Server](#rest-api-server-dravr-commere-server)
- [MCP Server](#mcp-server-dravr-commere-mcp)
- [Library Usage](#library-usage-rust)
- [Data Models](#data-models)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [License](#license)

## Quick Start

### Library (Rust)

```toml
[dependencies]
dravr-commere = { git = "https://github.com/dravr-ai/dravr-commere.git", tag = "v0.0.1" }
```

```rust
use dravr_commere::{NotificationService, DispatchRequest, TenantId};
use dravr_commere::models::NotificationCategory;
use serde_json::json;
use uuid::Uuid;

// Create service backed by SQLite
let service = NotificationService::from_sqlite(pool);

// Start the background scheduler (polls every 60s for cron-based notifications)
let abort_handle = service.start_scheduler();

// Dispatch a notification
let request = DispatchRequest {
    user_id: Uuid::new_v4(),
    tenant_id: TenantId::new(),
    category: NotificationCategory::Training,
    notification_type: "activity_synced".to_owned(),
    title: "New activity synced".to_owned(),
    body: "Run — 10.2 km in 52:14".to_owned(),
    data: Some(json!({ "screen": "activity", "id": "abc123" })),
    image_url: None,
    actions: None,
    bypass_frequency_cap: false,
};
let outcome = service.dispatch(&request).await?;
```

### REST API Server

```bash
cargo run --bin dravr-commere-server -- serve --port 3200
```

```bash
curl http://localhost:3200/health
# {"status":"ok","service":"dravr-commere","version":"0.0.1"}
```

### MCP Server (stdio)

```bash
cargo run --bin dravr-commere-mcp -- --transport stdio
```

Or over HTTP:

```bash
cargo run --bin dravr-commere-mcp -- --transport http --port 3200
```

## Dispatch Pipeline

Every notification passes through a three-step pipeline:

1. **Preference Check** — Is the category enabled? Within quiet hours? Daily frequency cap exceeded?
2. **Persistence** — Notification record saved to database (always, even if no devices)
3. **Push Delivery** — Expo Push API sends to all active device tokens (fire-and-forget)

Outcomes:

| Outcome | Description |
|---------|-------------|
| `Delivered` | Persisted and pushed to N devices |
| `PersistedNoDevices` | Persisted but user has no registered device tokens |
| `Suppressed(reason)` | Blocked by category disable, quiet hours, or frequency cap |

## Notification Categories

| Category | Description |
|----------|-------------|
| `Training` | Activity synced, training load alerts |
| `Recovery` | Low recovery score, overtraining warnings |
| `Social` | Friend requests, kudos, shared insights |
| `Coach` | Coach messages, plan updates, feedback (bypasses frequency cap) |
| `Achievement` | Personal records, milestones, fitness improvements |
| `System` | Sync failures, OAuth expiry |
| `Ai` | AI-generated insights and recommendations |
| `Reminders` | Scheduled workout reminders, weekly digests |

## Triggers

Fire-and-forget trigger functions spawn async tasks that dispatch notifications without blocking the caller. Failures are logged at WARN level.

| Trigger | Category | Description |
|---------|----------|-------------|
| `trigger_activity_synced` | Training | New activity from provider |
| `trigger_training_load_alert` | Training | ATL exceeds threshold |
| `trigger_low_recovery_score` | Recovery | Score dropped below threshold |
| `trigger_overtraining_warning` | Recovery | TSS trend suggests fatigue |
| `trigger_personal_record` | Achievement | New PR detected |
| `trigger_milestone_reached` | Achievement | Cumulative milestone |
| `trigger_fitness_improvement` | Achievement | FTP/VO2max improved |
| `trigger_friend_request_received` | Social | Accept/Decline action buttons |
| `trigger_friend_request_accepted` | Social | Connection confirmed |
| `trigger_activity_kudos` | Social | Someone gave kudos |
| `trigger_insight_shared` | Social | Coaching insight shared |
| `trigger_coach_message` | Coach | Quick Reply action button |
| `trigger_plan_updated` | Coach | Training plan changed |
| `trigger_coach_feedback` | Coach | Note on activity |
| `trigger_sync_failure` | System | Reconnect action button |

## Data Models

### NotificationService

The public facade encapsulating all notification operations:

- **Device tokens**: register, list, deactivate Expo push tokens
- **Preferences**: per-category enable/disable, quiet hours, frequency caps, timezone
- **Notifications**: create, list (with collapsing), mark read/opened/dismissed, analytics
- **Scheduled**: cron-based recurring notifications with timezone-aware evaluation
- **Dispatch**: full pipeline with preference checks and Expo Push delivery

### TenantId

Type-safe UUID newtype for multi-tenant isolation. Every database query includes `tenant_id` in the WHERE clause.

### Expo Push

HTTP client for the Expo Push API with batch sending (max 100 per request), receipt tracking, and structured error handling. Notifications are routed transparently to APNs (iOS) and FCM (Android).

## Configuration

All server configuration is loaded from environment variables (`.envrc` + direnv):

| Variable | Default | Description |
|----------|---------|-------------|
| `COMMERE_HOST` | `127.0.0.1` | Server bind address |
| `COMMERE_PORT` | `3200` | Server listen port |
| `COMMERE_TRANSPORT` | `http` | MCP transport mode (`stdio` or `http`) |
| `COMMERE_API_TOKEN` | *(none)* | Bearer token for REST API auth (empty = no auth) |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `NOTIFICATION_DEFAULT_MAX_PER_DAY` | `50` | Default daily cap per category |
| `NOTIFICATION_MAX_SCHEDULES_PER_USER` | `20` | Max scheduled notifications per user |
| `NOTIFICATION_FAN_OUT_PAGE_SIZE` | `100` | Batch size for friend fan-out queries |

Example `.envrc`:

```bash
export COMMERE_HOST="127.0.0.1"
export COMMERE_PORT="3200"
export RUST_LOG="dravr_commere=info,dravr_commere_mcp=info,dravr_commere_server=info"
export COMMERE_TRANSPORT="http"
export COMMERE_API_TOKEN=""
```

## Architecture

```
dravr-commere/
├── src/                           # Core library (models, dispatch, scheduling)
│   ├── lib.rs                     # Public API and module declarations
│   ├── error.rs                   # CommereError structured error types
│   ├── models.rs                  # TenantId, Notification, DeviceToken, Preferences, DTOs
│   ├── service.rs                 # NotificationService facade
│   ├── dispatch.rs                # Three-step dispatch pipeline
│   ├── triggers.rs                # Fire-and-forget trigger functions (17 triggers)
│   ├── scheduler.rs               # Cron-based background scheduler (60s poll)
│   ├── expo_push.rs               # Expo Push API client (batch send, receipts)
│   ├── constants.rs               # Configurable defaults (caps, page sizes)
│   └── repository/                # Database-agnostic trait + implementations
│       ├── mod.rs                 # NotificationRepository trait
│       ├── sqlite.rs              # SQLite backend (~1,100 LOC)
│       └── postgres.rs            # PostgreSQL backend (~1,100 LOC)
│
├── crates/
│   ├── dravr-commere-mcp/         # MCP server (library + binary crate, powered by dravr-tronc)
│   │   ├── src/state.rs           # SharedState with notification config
│   │   └── src/tools/             # Domain-specific MCP tool implementations
│   │
│   └── dravr-commere-server/      # Unified REST API + MCP server (binary crate, powered by dravr-tronc)
│       ├── src/router.rs          # Axum routes (/health, /mcp)
│       ├── src/auth.rs            # Bearer token middleware
│       └── src/main.rs            # CLI (serve, stdio)
│
└── tests/                         # Integration tests
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
