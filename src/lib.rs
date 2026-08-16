// ABOUTME: Multi-tenant push notification service with dispatch pipeline and scheduling
// ABOUTME: Provides NotificationService facade encapsulating persistence, dispatch, and Expo Push
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

//! # dravr-commere
//!
//! Standalone multi-tenant push notification service. Owns its own models,
//! error types, persistence, dispatch pipeline, and scheduling.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dravr_commere::service::NotificationService;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // SQLite backend
//! let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
//! let service = NotificationService::from_sqlite(pool);
//!
//! // PostgreSQL backend takes a PgPool the same way:
//! //   let service = NotificationService::from_postgres(pg_pool);
//!
//! // Start the background scheduler
//! let abort_handle = service.start_scheduler();
//! # let _ = abort_handle;
//! # Ok(())
//! # }
//! ```

pub mod constants;
pub mod error;
pub mod expo_push;
pub mod models;
pub mod service;
pub mod triggers;

pub(crate) mod dispatch;
pub(crate) mod repository;
pub(crate) mod scheduler;

// Re-export primary public types at crate root
pub use dispatch::{DispatchOutcome, DispatchRequest, SuppressionReason};
pub use error::{CommereError, CommereResult};
pub use models::TenantId;
pub use scheduler::{compute_next_fire_time, validate_cron_expression};
pub use service::NotificationService;
