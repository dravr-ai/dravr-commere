#!/bin/bash
# ABOUTME: Pre-push validation script for dravr-commere
# ABOUTME: Runs fmt, clippy, and tests, then creates a validation marker
#
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 dravr.ai

set -e

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
GIT_DIR="$(git rev-parse --git-dir)"
MARKER_FILE="$GIT_DIR/validation-passed"

echo "Running pre-push validation..."
echo ""

echo "Tier 0: Format check..."
cargo fmt --all -- --check
echo "  ✅ Format OK"

echo "Tier 1: Clippy..."
cargo clippy --workspace --all-targets --quiet -- -D warnings
echo "  ✅ Clippy OK"

echo "Tier 2: Tests..."
cargo test --workspace --quiet
echo "  ✅ Tests OK"

# Create marker
CURRENT_COMMIT=$(git rev-parse HEAD)
CURRENT_TIMESTAMP=$(date +%s)
echo "${CURRENT_TIMESTAMP} ${CURRENT_COMMIT}" > "$MARKER_FILE"

echo ""
echo "✅ All validations passed"
echo "   Marker created: ${MARKER_FILE}"
echo "   Commit: ${CURRENT_COMMIT:0:8}"
echo ""
echo "You can now push: git push"
