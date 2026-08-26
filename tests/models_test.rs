// ABOUTME: Round-trip and totality tests for the persisted enums and the tenant newtype
// ABOUTME: These strings are a storage format — a silent rename is a data-migration bug
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(missing_docs)]

//! `NotificationCategory::as_str` and `DevicePlatform::as_str` are written into
//! the database, and `from_str_opt` reads them back. The pair has to be exactly
//! inverse: a category whose string changes silently orphans every stored row
//! carrying the old spelling, and one missing from `all()` is invisible to any
//! caller that enumerates categories to build preferences.

use dravr_commere::models::{DevicePlatform, NotificationCategory, TenantId};
use uuid::Uuid;

#[test]
fn every_category_round_trips_through_its_stored_string() {
    for category in NotificationCategory::all() {
        let stored = category.as_str();
        assert_eq!(
            NotificationCategory::from_str_opt(stored),
            Some(*category),
            "`{stored}` must read back as the category that wrote it"
        );
    }
}

#[test]
fn all_lists_every_category_exactly_once() {
    let all = NotificationCategory::all();
    let mut seen: Vec<&str> = all.iter().map(NotificationCategory::as_str).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), before, "all() must not repeat a category");

    // A category absent from all() is unreachable for anything that enumerates
    // them (preference screens, fan-out), so pin the count against the seven
    // documented kinds rather than against all().len(), which would agree with
    // itself no matter what.
    assert_eq!(
        before, 7,
        "expected the seven documented categories, got: {seen:?}"
    );
}

#[test]
fn the_retired_social_category_no_longer_parses() {
    // `social` was a stored category until 0.2.0. dravr-platform retired the
    // Insights and Friends surfaces that raised it and its migration deleted
    // the stored rows, so a reader that still resolved the string would
    // resurrect a category no preference screen can render.
    assert_eq!(NotificationCategory::from_str_opt("social"), None);
    assert!(
        NotificationCategory::all()
            .iter()
            .all(|category| category.as_str() != "social"),
        "all() must not enumerate the retired social category"
    );
}

#[test]
fn an_unknown_category_string_is_rejected_not_defaulted() {
    for bogus in ["", "Training", "training ", "nope"] {
        assert_eq!(
            NotificationCategory::from_str_opt(bogus),
            None,
            "`{bogus}` is not a stored category and must not resolve to one"
        );
    }
}

#[test]
fn every_platform_round_trips_and_rejects_unknowns() {
    for platform in [DevicePlatform::Ios, DevicePlatform::Android] {
        assert_eq!(
            DevicePlatform::from_str_opt(platform.as_str()),
            Some(platform),
            "platform `{}` must read back",
            platform.as_str()
        );
    }
    for bogus in ["", "iOS", "web", "windows"] {
        assert_eq!(
            DevicePlatform::from_str_opt(bogus),
            None,
            "`{bogus}` is not a platform"
        );
    }
}

#[test]
fn tenant_id_parses_from_its_own_rendering() {
    let id = TenantId::new();
    let parsed: TenantId = id.as_uuid().to_string().parse().unwrap();
    assert_eq!(parsed.as_uuid(), id.as_uuid());

    assert_eq!(TenantId::nil().as_uuid(), Uuid::nil());
    assert!(
        "not-a-uuid".parse::<TenantId>().is_err(),
        "a malformed tenant id must fail rather than resolve to nil, which would \
         read as a real tenant"
    );
}

#[test]
fn two_generated_tenant_ids_differ() {
    assert_ne!(
        TenantId::new().as_uuid(),
        TenantId::new().as_uuid(),
        "TenantId::new must generate, not return a constant"
    );
}
