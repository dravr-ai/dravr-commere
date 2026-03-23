// ABOUTME: NotificationService facade — the single public entry point for all notification operations
// ABOUTME: Encapsulates persistence (SQLite/PostgreSQL), dispatch pipeline, and cron scheduling
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

//! # Notification Service
//!
//! `NotificationService` is the public facade for the notification subsystem.
//! Consumers construct it via `from_sqlite()` or `from_postgres()` and interact
//! through its methods — the underlying `NotificationRepository` is never exposed.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::dispatch::{DispatchOutcome, DispatchRequest, NotificationDispatcher};
use crate::error::CommereResult;
use crate::expo_push::ExpoPushService;
use crate::models::{
    CreateNotificationParams, CreateScheduledNotificationParams, DeviceToken, Notification,
    NotificationAnalytics, NotificationPreference, ScheduledNotification, TenantId,
    UpdateScheduledNotificationParams, UpsertNotificationPreferenceParams,
};
use crate::repository::NotificationRepository;
use crate::scheduler;

/// Public facade for the notification subsystem
///
/// Owns the database repository, dispatch pipeline, and scheduler lifecycle.
/// Constructed via `from_sqlite()` or `from_postgres()`.
pub struct NotificationService {
    /// Database-agnostic notification repository
    repo: Arc<dyn NotificationRepository>,
    /// Notification dispatcher (preference checks, persistence, push delivery)
    dispatcher: Arc<NotificationDispatcher>,
}

impl NotificationService {
    /// Create a `NotificationService` backed by a `SQLite` database pool
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn from_sqlite(pool: sqlx::SqlitePool) -> Self {
        use crate::repository::sqlite::SqliteNotificationRepository;

        let repo: Arc<dyn NotificationRepository> =
            Arc::new(SqliteNotificationRepository::new(pool));
        let expo_push = Arc::new(ExpoPushService::new());
        let dispatcher = Arc::new(NotificationDispatcher::new(Arc::clone(&repo), expo_push));
        Self { repo, dispatcher }
    }

    /// Create a `NotificationService` backed by a `PostgreSQL` database pool
    #[cfg(feature = "postgresql")]
    #[must_use]
    pub fn from_postgres(pool: sqlx::PgPool) -> Self {
        use crate::repository::postgres::PostgresNotificationRepository;

        let repo: Arc<dyn NotificationRepository> =
            Arc::new(PostgresNotificationRepository::new(pool));
        let expo_push = Arc::new(ExpoPushService::new());
        let dispatcher = Arc::new(NotificationDispatcher::new(Arc::clone(&repo), expo_push));
        Self { repo, dispatcher }
    }

    /// Start the background scheduler that polls for due scheduled notifications.
    ///
    /// Returns an `AbortHandle` to stop the scheduler on shutdown.
    #[must_use]
    pub fn start_scheduler(&self) -> AbortHandle {
        scheduler::start_notification_scheduler(
            Arc::clone(&self.repo),
            Arc::clone(&self.dispatcher),
        )
    }

    // ── Dispatch ──

    /// Dispatch a notification through the full pipeline (preference checks, persist, push)
    pub async fn dispatch(&self, request: &DispatchRequest) -> CommereResult<DispatchOutcome> {
        self.dispatcher.dispatch(request).await
    }

    // ── Device Tokens ──

    /// Register or update a device token for push notifications
    pub async fn upsert_device_token(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        expo_push_token: &str,
        platform: &str,
        device_name: Option<&str>,
    ) -> CommereResult<DeviceToken> {
        self.repo
            .upsert_device_token(user_id, tenant_id, expo_push_token, platform, device_name)
            .await
    }

    /// Get all active device tokens for a user within a tenant
    pub async fn get_device_tokens(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<Vec<DeviceToken>> {
        self.repo.get_device_tokens(user_id, tenant_id).await
    }

    /// Deactivate a device token (soft-delete)
    pub async fn deactivate_device_token(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        token_id: Uuid,
    ) -> CommereResult<bool> {
        self.repo
            .deactivate_device_token(user_id, tenant_id, token_id)
            .await
    }

    // ── Notification Preferences ──

    /// Get all notification preferences for a user within a tenant
    pub async fn get_notification_preferences(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<Vec<NotificationPreference>> {
        self.repo
            .get_notification_preferences(user_id, tenant_id)
            .await
    }

    /// Upsert a notification preference for a specific category
    pub async fn upsert_notification_preference(
        &self,
        params: &UpsertNotificationPreferenceParams,
    ) -> CommereResult<NotificationPreference> {
        self.repo.upsert_notification_preference(params).await
    }

    // ── Notifications ──

    /// Create a new notification record
    pub async fn create_notification(
        &self,
        params: &CreateNotificationParams,
    ) -> CommereResult<Notification> {
        self.repo.create_notification(params).await
    }

    /// List notifications for a user with optional filters.
    /// Returns (notifications, `total_count`, `unread_count`).
    pub async fn list_notifications(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        limit: u32,
        offset: u32,
        category: Option<&str>,
        unread_only: bool,
    ) -> CommereResult<(Vec<Notification>, i64, i64)> {
        self.repo
            .list_notifications(user_id, tenant_id, limit, offset, category, unread_only)
            .await
    }

    /// Mark a notification as read
    pub async fn mark_notification_read(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        notification_id: Uuid,
    ) -> CommereResult<bool> {
        self.repo
            .mark_notification_read(user_id, tenant_id, notification_id)
            .await
    }

    /// Mark all notifications as read for a user within a tenant
    pub async fn mark_all_notifications_read(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<u64> {
        self.repo
            .mark_all_notifications_read(user_id, tenant_id)
            .await
    }

    /// Delete a notification
    pub async fn delete_notification(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        notification_id: Uuid,
    ) -> CommereResult<bool> {
        self.repo
            .delete_notification(user_id, tenant_id, notification_id)
            .await
    }

    /// Get unread notification count for a user within a tenant
    pub async fn get_unread_count(&self, user_id: Uuid, tenant_id: TenantId) -> CommereResult<i64> {
        self.repo.get_unread_count(user_id, tenant_id).await
    }

    /// Count notifications of a specific category created since a given timestamp
    pub async fn count_notifications_since(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        category: &str,
        since: DateTime<Utc>,
    ) -> CommereResult<i64> {
        self.repo
            .count_notifications_since(user_id, tenant_id, category, since)
            .await
    }

    // ── Notification Analytics ──

    /// Record when a user opened a notification
    pub async fn mark_notification_opened(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        notification_id: Uuid,
    ) -> CommereResult<bool> {
        self.repo
            .mark_notification_opened(user_id, tenant_id, notification_id)
            .await
    }

    /// Record when a user dismissed a notification
    pub async fn mark_notification_dismissed(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        notification_id: Uuid,
    ) -> CommereResult<bool> {
        self.repo
            .mark_notification_dismissed(user_id, tenant_id, notification_id)
            .await
    }

    /// Get notification analytics for a specific user within a tenant
    pub async fn get_notification_analytics(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        category: Option<&str>,
    ) -> CommereResult<NotificationAnalytics> {
        self.repo
            .get_notification_analytics(user_id, tenant_id, since, until, category)
            .await
    }

    // ── Scheduled Notifications ──

    /// Get a single scheduled notification by ID (tenant-isolated)
    pub async fn get_scheduled_notification_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<Option<ScheduledNotification>> {
        self.repo
            .get_scheduled_notification_by_id(id, user_id, tenant_id)
            .await
    }

    /// Count scheduled notifications for a user within a tenant
    pub async fn count_scheduled_notifications(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<i64> {
        self.repo
            .count_scheduled_notifications(user_id, tenant_id)
            .await
    }

    /// Create a scheduled notification
    pub async fn create_scheduled_notification(
        &self,
        params: &CreateScheduledNotificationParams,
    ) -> CommereResult<ScheduledNotification> {
        self.repo.create_scheduled_notification(params).await
    }

    /// List scheduled notifications for a user within a tenant
    pub async fn list_scheduled_notifications(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<Vec<ScheduledNotification>> {
        self.repo
            .list_scheduled_notifications(user_id, tenant_id)
            .await
    }

    /// Delete a scheduled notification (tenant-isolated)
    pub async fn delete_scheduled_notification(
        &self,
        id: Uuid,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> CommereResult<bool> {
        self.repo
            .delete_scheduled_notification(id, user_id, tenant_id)
            .await
    }

    /// Update a scheduled notification (enable/disable, change cron/timezone)
    pub async fn update_scheduled_notification(
        &self,
        params: &UpdateScheduledNotificationParams,
    ) -> CommereResult<bool> {
        self.repo.update_scheduled_notification(params).await
    }
}
