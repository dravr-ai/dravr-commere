// ABOUTME: The error type's constructors, conversions and operator-facing messages
// ABOUTME: A validation failure must never be reported as a database failure
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(missing_docs)]

//! These variants are what an operator reads when a push fails to land. The
//! distinction between them is the diagnosis: `PushDelivery` points at Expo,
//! `Database` at us, `Validation` at the caller. Collapsing any into another
//! sends whoever is on call to the wrong place, so the tests assert the variant
//! *and* that the detail handed in survives into the message.

use dravr_commere::error::CommereError;

#[test]
fn each_constructor_produces_its_own_variant() {
    assert!(matches!(
        CommereError::database("connection reset"),
        CommereError::Database(_)
    ));
    assert!(matches!(
        CommereError::push_delivery("expo", "DeviceNotRegistered"),
        CommereError::PushDelivery { .. }
    ));
    assert!(matches!(
        CommereError::validation("token", "empty"),
        CommereError::Validation { .. }
    ));
    assert!(matches!(
        CommereError::not_found("device"),
        CommereError::NotFound { .. }
    ));
}

#[test]
fn the_detail_handed_in_survives_into_the_message() {
    let e = CommereError::push_delivery("expo", "DeviceNotRegistered");
    let text = e.to_string();
    assert!(
        text.contains("expo") && text.contains("DeviceNotRegistered"),
        "the service and the upstream reason are the whole diagnostic value, got: {text}"
    );

    let v = CommereError::validation("expo_token", "must start with ExponentPushToken");
    let vtext = v.to_string();
    assert!(
        vtext.contains("expo_token") && vtext.contains("ExponentPushToken"),
        "a validation error must name the field and the rule, got: {vtext}"
    );

    let nf = CommereError::not_found("schedule 42");
    assert!(
        nf.to_string().contains("schedule 42"),
        "not-found must name the resource, got: {nf}"
    );
}

#[test]
fn a_malformed_uuid_is_a_validation_error_not_a_database_error() {
    let err: CommereError = "not-a-uuid".parse::<uuid::Uuid>().unwrap_err().into();
    match err {
        CommereError::Validation { field, reason } => {
            assert_eq!(field, "uuid");
            assert!(
                !reason.is_empty(),
                "the parse reason must be carried, not dropped"
            );
        }
        other => panic!("a caller's bad uuid is their fault, not the database's: {other:?}"),
    }
}

#[test]
fn a_sqlx_failure_becomes_a_database_error_carrying_its_text() {
    let err: CommereError = sqlx::Error::RowNotFound.into();
    match err {
        CommereError::Database(text) => assert!(
            !text.is_empty(),
            "the sqlx message is the only clue to what failed; it must not be discarded"
        ),
        other => panic!("expected Database, got {other:?}"),
    }
}
