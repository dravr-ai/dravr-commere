// ABOUTME: Structured error types for the notification service
// ABOUTME: Covers database, push delivery, validation, and scheduling failures
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

//! Notification service error types
//!
//! All errors are structured variants — no `anyhow!` or ad-hoc string errors.

use std::fmt::Display;

use thiserror::Error;

/// Result type alias for notification operations
pub type CommereResult<T> = Result<T, CommereError>;

/// Structured error type for the notification service
#[derive(Debug, Error)]
pub enum CommereError {
    /// Database operation failed
    #[error("database error: {0}")]
    Database(String),

    /// Push delivery failed (Expo API)
    #[error("push delivery error ({service}): {message}")]
    PushDelivery {
        /// External service name (e.g., "Expo")
        service: String,
        /// Error description
        message: String,
    },

    /// Input validation failed
    #[error("validation error: {field} — {reason}")]
    Validation {
        /// Field that failed validation
        field: String,
        /// Reason for failure
        reason: String,
    },

    /// Scheduling error (invalid cron, timezone, etc.)
    #[error("scheduling error: {0}")]
    Scheduling(String),

    /// Resource not found
    #[error("{resource} not found")]
    NotFound {
        /// Resource description
        resource: String,
    },
}

impl CommereError {
    /// Create a database error from any displayable source
    pub fn database(source: impl Display) -> Self {
        Self::Database(source.to_string())
    }

    /// Create a push delivery error
    pub fn push_delivery(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PushDelivery {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Create a not-found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
}

impl From<sqlx::Error> for CommereError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<uuid::Error> for CommereError {
    fn from(err: uuid::Error) -> Self {
        Self::Validation {
            field: "uuid".to_owned(),
            reason: err.to_string(),
        }
    }
}
