// ABOUTME: Health check endpoint returning server status
// ABOUTME: Simple GET /health handler for monitoring and readiness probes
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use axum::response::IntoResponse;
use dravr_tronc::server::health::HealthResponse;

/// Health check handler returning server status
pub async fn health_check() -> impl IntoResponse {
    HealthResponse::ok("dravr-commere", env!("CARGO_PKG_VERSION")).into_axum_response()
}
