// ABOUTME: Push notification models for device tokens, preferences, and notification records
// ABOUTME: Multi-tenant notification delivery with per-user category controls and quiet hours
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// TenantId — type-safe wrapper for multi-tenant isolation
// ============================================================================

/// Type-safe wrapper for tenant identifiers
///
/// Provides compile-time distinction between tenant IDs and other UUIDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(pub Uuid);

impl TenantId {
    /// Create a new random `TenantId`
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a `TenantId` from a UUID
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID value
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Create a nil (all zeros) `TenantId`
    #[must_use]
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for TenantId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<TenantId> for Uuid {
    fn from(tenant_id: TenantId) -> Self {
        tenant_id.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TenantId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl AsRef<Uuid> for TenantId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(feature = "sqlite")]
mod sqlite_impl {
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;
    use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
    use sqlx::{Decode, Encode, Type};
    use uuid::Uuid;

    use super::TenantId;

    impl Type<Sqlite> for TenantId {
        fn type_info() -> SqliteTypeInfo {
            <String as Type<Sqlite>>::type_info()
        }
    }

    impl<'q> Encode<'q, Sqlite> for TenantId {
        fn encode_by_ref(
            &self,
            buf: &mut Vec<SqliteArgumentValue<'q>>,
        ) -> Result<IsNull, BoxDynError> {
            let text = self.0.to_string();
            <String as Encode<'q, Sqlite>>::encode(text, buf)
        }
    }

    impl<'r> Decode<'r, Sqlite> for TenantId {
        fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
            let text = <String as Decode<'r, Sqlite>>::decode(value)?;
            let uuid = Uuid::parse_str(&text)?;
            Ok(Self(uuid))
        }
    }
}

#[cfg(feature = "postgresql")]
mod postgres_impl {
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};
    use sqlx::{Decode, Encode, Type};
    use uuid::Uuid;

    use super::TenantId;

    impl Type<Postgres> for TenantId {
        fn type_info() -> PgTypeInfo {
            <Uuid as Type<Postgres>>::type_info()
        }
    }

    impl<'q> Encode<'q, Postgres> for TenantId {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            <Uuid as Encode<'q, Postgres>>::encode_by_ref(&self.0, buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for TenantId {
        fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
            let uuid = <Uuid as Decode<'r, Postgres>>::decode(value)?;
            Ok(Self(uuid))
        }
    }
}

// ============================================================================
// Notification Categories
// ============================================================================

/// Supported notification categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    /// Training activity notifications (synced, load alerts)
    Training,
    /// Recovery score and overtraining alerts
    Recovery,
    /// Social interactions (friend requests, kudos)
    Social,
    /// Coach messages and plan updates
    Coach,
    /// Personal records and milestones
    Achievement,
    /// System notifications (OAuth expiry, sync failure)
    System,
    /// AI-generated insights and recommendations
    Ai,
    /// Scheduled reminders
    Reminders,
}

impl NotificationCategory {
    /// Return all valid categories
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Training,
            Self::Recovery,
            Self::Social,
            Self::Coach,
            Self::Achievement,
            Self::System,
            Self::Ai,
            Self::Reminders,
        ]
    }

    /// String representation for database storage
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Training => "training",
            Self::Recovery => "recovery",
            Self::Social => "social",
            Self::Coach => "coach",
            Self::Achievement => "achievement",
            Self::System => "system",
            Self::Ai => "ai",
            Self::Reminders => "reminders",
        }
    }

    /// Parse from string, returning None for unrecognized values
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "training" => Some(Self::Training),
            "recovery" => Some(Self::Recovery),
            "social" => Some(Self::Social),
            "coach" => Some(Self::Coach),
            "achievement" => Some(Self::Achievement),
            "system" => Some(Self::System),
            "ai" => Some(Self::Ai),
            "reminders" => Some(Self::Reminders),
            _ => None,
        }
    }
}

impl fmt::Display for NotificationCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// Device Platform
// ============================================================================

/// Mobile device platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevicePlatform {
    /// Apple iOS
    Ios,
    /// Google Android
    Android,
}

impl DevicePlatform {
    /// String representation for database storage
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }

    /// Parse from string
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "ios" => Some(Self::Ios),
            "android" => Some(Self::Android),
            _ => None,
        }
    }
}

impl fmt::Display for DevicePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================================
// Device Token
// ============================================================================

/// Registered device for push notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    /// Unique device token identifier
    pub id: Uuid,
    /// User who owns this device
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Expo push token string (e.g. `ExponentPushToken[...]`)
    pub expo_push_token: String,
    /// Device platform (ios or android)
    pub platform: DevicePlatform,
    /// Human-readable device name
    pub device_name: Option<String>,
    /// Whether this token is active for receiving notifications
    pub active: bool,
    /// When the token was first registered
    pub created_at: DateTime<Utc>,
    /// When the token was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request to register or update a device token
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDeviceTokenRequest {
    /// Expo push token string
    pub expo_push_token: String,
    /// Device platform
    pub platform: DevicePlatform,
    /// Human-readable device name
    pub device_name: Option<String>,
}

// ============================================================================
// Notification Preferences
// ============================================================================

/// Per-category notification preference for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    /// Unique preference identifier
    pub id: Uuid,
    /// User who owns this preference
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Notification category this preference controls
    pub category: NotificationCategory,
    /// Whether notifications in this category are enabled
    pub enabled: bool,
    /// Granular per-type toggles within the category
    pub sub_preferences: Option<serde_json::Value>,
    /// Quiet hours start time (no notifications sent during quiet hours)
    pub quiet_hours_start: Option<NaiveTime>,
    /// Quiet hours end time
    pub quiet_hours_end: Option<NaiveTime>,
    /// Timezone for quiet hours calculation
    pub timezone: Option<String>,
    /// Maximum notifications per day for this category
    pub max_per_day: Option<i32>,
    /// When the preference was created
    pub created_at: DateTime<Utc>,
    /// When the preference was last updated
    pub updated_at: DateTime<Utc>,
}

/// Parameters for upserting a notification preference in the database
#[derive(Debug, Clone)]
pub struct UpsertNotificationPreferenceParams {
    /// User who owns this preference
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Category to update
    pub category: String,
    /// Whether notifications are enabled
    pub enabled: bool,
    /// Granular per-type toggles
    pub sub_preferences: Option<serde_json::Value>,
    /// Quiet hours start (HH:MM format)
    pub quiet_hours_start: Option<String>,
    /// Quiet hours end (HH:MM format)
    pub quiet_hours_end: Option<String>,
    /// Timezone for quiet hours
    pub timezone: Option<String>,
    /// Maximum notifications per day
    pub max_per_day: Option<i32>,
}

// ============================================================================
// Notification Actions
// ============================================================================

/// Type of action a notification button can trigger
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationActionType {
    /// Navigate to a specific screen
    OpenScreen,
    /// Show accept/decline buttons (e.g., friend requests)
    AcceptDecline,
    /// Show a quick reply input (e.g., coach messages)
    QuickReply,
}

/// An action button attached to a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Unique identifier for this action within the notification
    pub id: String,
    /// Button label displayed to the user
    pub title: String,
    /// Type of action triggered when the button is tapped
    pub action_type: NotificationActionType,
}

// ============================================================================
// Notification Record
// ============================================================================

/// A notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification identifier
    pub id: Uuid,
    /// Recipient user
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Notification category (training, recovery, social, etc.)
    pub category: NotificationCategory,
    /// Specific notification type within the category (e.g. `activity_synced`)
    pub notification_type: String,
    /// Notification title
    pub title: String,
    /// Notification body text
    pub body: String,
    /// JSON payload for deep-link routing and action data
    pub data: Option<serde_json::Value>,
    /// Optional image URL for rich notifications
    pub image_url: Option<String>,
    /// When the notification was read (None if unread)
    pub read_at: Option<DateTime<Utc>>,
    /// When the push notification was delivered to the device
    pub delivered_at: Option<DateTime<Utc>>,
    /// When the user opened the notification
    pub opened_at: Option<DateTime<Utc>>,
    /// When the user dismissed the notification
    pub dismissed_at: Option<DateTime<Utc>>,
    /// Action buttons attached to this notification
    pub actions: Option<Vec<NotificationAction>>,
    /// When the notification was created
    pub created_at: DateTime<Utc>,
}

/// Parameters for creating a new notification
#[derive(Debug, Clone)]
pub struct CreateNotificationParams {
    /// Recipient user
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Notification category
    pub category: NotificationCategory,
    /// Specific notification type within the category
    pub notification_type: String,
    /// Notification title
    pub title: String,
    /// Notification body text
    pub body: String,
    /// JSON payload for deep-link routing
    pub data: Option<serde_json::Value>,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Action buttons attached to this notification
    pub actions: Option<Vec<NotificationAction>>,
}

// ============================================================================
// Scheduled Notifications
// ============================================================================

/// A scheduled notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledNotification {
    /// Unique scheduled notification identifier
    pub id: Uuid,
    /// Target user
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Notification type to send on each trigger
    pub notification_type: String,
    /// Cron expression defining the schedule
    pub schedule_cron: String,
    /// Timezone for cron evaluation
    pub timezone: String,
    /// Pre-computed next fire timestamp
    pub next_fire_at: Option<DateTime<Utc>>,
    /// Whether this schedule is active
    pub enabled: bool,
    /// When this schedule last triggered
    pub last_fired_at: Option<DateTime<Utc>>,
    /// When the schedule was created
    pub created_at: DateTime<Utc>,
}

/// Parameters for creating a new scheduled notification
#[derive(Debug, Clone)]
pub struct CreateScheduledNotificationParams {
    /// Target user
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Notification type to send on each trigger (e.g. `weekly_training_summary`)
    pub notification_type: String,
    /// Cron expression (validated before storage)
    pub schedule_cron: String,
    /// Timezone for cron evaluation (IANA timezone name)
    pub timezone: String,
    /// Pre-computed next fire timestamp (computed from cron + timezone)
    pub next_fire_at: DateTime<Utc>,
}

/// Parameters for updating a scheduled notification in the database
#[derive(Debug, Clone)]
pub struct UpdateScheduledNotificationParams {
    /// Schedule ID to update
    pub id: Uuid,
    /// Owner user ID (for authorization)
    pub user_id: Uuid,
    /// Tenant scope for multi-tenant isolation
    pub tenant_id: TenantId,
    /// Whether the schedule should be enabled
    pub enabled: Option<bool>,
    /// Updated cron expression
    pub schedule_cron: Option<String>,
    /// Updated timezone
    pub timezone: Option<String>,
    /// Recomputed next fire timestamp
    pub next_fire_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Analytics
// ============================================================================

/// Aggregated notification analytics for a tenant
#[derive(Debug, Clone, Serialize)]
pub struct NotificationAnalytics {
    /// Total notifications sent in the period
    pub total_sent: i64,
    /// Fraction of sent that were delivered (0.0-1.0)
    pub delivery_rate: f64,
    /// Fraction of delivered that were opened (0.0-1.0)
    pub open_rate: f64,
    /// Fraction of delivered that were dismissed (0.0-1.0)
    pub dismiss_rate: f64,
    /// Breakdown by notification category
    pub category_breakdown: Vec<CategoryAnalytics>,
    /// Average seconds between delivery and open
    pub avg_time_to_open_seconds: Option<f64>,
}

/// Per-category analytics breakdown
#[derive(Debug, Clone, Serialize)]
pub struct CategoryAnalytics {
    /// Category name
    pub category: String,
    /// Number sent in this category
    pub sent: i64,
    /// Number opened
    pub opened: i64,
    /// Number dismissed
    pub dismissed: i64,
}
