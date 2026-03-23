// ABOUTME: Shared server state for the MCP server
// ABOUTME: Thread-safe state container holding notification service configuration
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use tokio::sync::RwLock;

/// Type alias for the shared state handle used across the server
pub type SharedState = Arc<RwLock<ServerState>>;

/// Server state holding notification service configuration
#[derive(Debug)]
pub struct ServerState {
    /// Whether the server has been initialized via MCP initialize handshake
    initialized: bool,
}

impl ServerState {
    /// Create a new server state
    #[must_use]
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Mark the server as initialized
    pub fn set_initialized(&mut self) {
        self.initialized = true;
    }

    /// Check if the server has been initialized
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
