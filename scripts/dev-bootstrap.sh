#!/usr/bin/env bash
# Dev bootstrap: check toolchain and build the workspace.
set -euo pipefail

echo "[dev-bootstrap] checking rustc..."
if ! command -v rustc >/dev/null 2>&1; then
    echo "rustc not found. Install rustup: https://rustup.rs/" >&2
    exit 1
fi

RUSTC_VERSION=$(rustc --version | awk '{print $2}')
REQUIRED="1.75.0"
if ! printf '%s\n%s\n' "$REQUIRED" "$RUSTC_VERSION" | sort -V -C; then
    echo "rustc $RUSTC_VERSION < $REQUIRED. Please update: rustup update stable" >&2
    exit 1
fi

echo "[dev-bootstrap] rustc $RUSTC_VERSION OK"

echo "[dev-bootstrap] checking cargo..."
command -v cargo >/dev/null 2>&1 || { echo "cargo not found" >&2; exit 1; }

echo "[dev-bootstrap] building workspace..."
cargo build --workspace

echo "[dev-bootstrap] OK"
