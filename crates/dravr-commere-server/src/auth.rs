// ABOUTME: Bearer token authentication middleware for REST API
// ABOUTME: Delegates to dravr-tronc shared auth with COMMERE_API_TOKEN env var
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use dravr_tronc::server::auth;

/// Environment variable name for the API key
const API_KEY_ENV: &str = "COMMERE_API_TOKEN";

/// Bearer token authentication middleware.
///
/// Delegates to `dravr_tronc::server::auth::require_auth` with the
/// `COMMERE_API_TOKEN` environment variable. If the variable is not set,
/// all requests pass through (development mode).
pub async fn require_auth(request: Request, next: Next) -> Response {
    auth::require_auth(API_KEY_ENV, request, next).await
}
