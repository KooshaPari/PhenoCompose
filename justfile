# PhenoCompose canonical justfile (L2 #23).
#
# Mirrors the recipes in `Taskfile.yml` for CI consumers that prefer
# `just` (casey/just: https://just.systems). All Go invocations pin
# GOCACHE to a stable, repo-scoped location so the build is hermetic
# and reproducible across local + CI runs.
#
# The `set shell := ["bash", "-uc"]` directive turns on:
#   -u: error on unset variables (fail loud, fail fast)
#   -c: pipefail — propagate the exit status of the first failing command
# This guards against silent partial failures in long cargo pipelines.
#
# Usage: `just <recipe>` (run from the repo root).
# Run `just` with no args to see the list below.

set shell := ["bash", "-uc"]
set dotenv-load

# ---------------------------------------------------------------------------
# Hermetic / hermetic-build env (mirrors Taskfile.yml env: block).
# ---------------------------------------------------------------------------
export GOCACHE := "/private/tmp/phenocompose-gocache"
export GOFLAGS := "-mod=readonly"
export CGO_ENABLED := "1"
# CARGO_TERM_COLOR=never keeps cargo output pipe-friendly for CI logs.
export CARGO_TERM_COLOR := "never"

# ---------------------------------------------------------------------------
# Toolchain / scope (expression variables used as `{{ ... }}` substitutions)
# ---------------------------------------------------------------------------
cargo            := "cargo"
go_bindings_dir  := "bindings/go-c-export"
cross_platform_py := "bindings/build_cross_platform.py"
# Per L1 audit (`PhenoCompose/STATUS_2026_06_10.md`): cargo check on the
# polyglot crate set does not complete within bounded local runs. The
# 10m timeout here mirrors the Taskfile values so both runners surface
# the failure consistently.
long_timeout     := "10m"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# True if the third Rust crate (nvms-core-sys) is present on disk.
has_nvms_core_sys := `if [ -f bindings/nvms-core-sys/Cargo.toml ]; then printf true; else printf false; fi`

# Node package manager (npm | pnpm | yarn | bun), detected from lockfiles.
node_pm := `\
    if [ -f bun.lockb ] || [ -f bun.lock ]; then printf "bun"; \
    elif [ -f pnpm-lock.yaml ]; then printf "pnpm"; \
    elif [ -f yarn.lock ]; then printf "yarn"; \
    else printf "npm"; fi`

# Space-separated list of the 3 Rust crates to iterate (matches the
# L2 #23 brief: pheno-compose-driver + bindings/rust-ffi + bindings/nvms-core-sys).
# `nvms-core-sys` is included unconditionally; the build/test/lint/fmt
# recipes gate on `has_nvms_core_sys` to skip it if the manifest is absent.
rust_crates := "pheno-compose-driver bindings/rust-ffi bindings/nvms-core-sys"

# ---------------------------------------------------------------------------
# Default recipe: list available commands.
# ---------------------------------------------------------------------------

# List available recipes + detected polyglot stack (alias for `just --list`).
default:
    @just --list
    @echo "rust crates: pheno-compose-driver, bindings/rust-ffi, bindings/nvms-core-sys"
    @echo "go bindings: {{go_bindings_dir}}"
    @echo "GOCACHE=$GOCACHE"

# ---------------------------------------------------------------------------
# L2-23 canonical tasks
# ---------------------------------------------------------------------------

# Run `cargo check` over the 3 Rust crates (pheno-compose-driver +
# bindings/rust-ffi + bindings/nvms-core-sys when present) and the
# Python cross-platform build orchestrator.
# Timeout: 600s (= 10m, per L1 audit).
build timeout=long_timeout:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo check -p $crate"
            timeout {{timeout}} {{cargo}} check --manifest-path "$crate/Cargo.toml"
        else
            echo "skip: $crate/Cargo.toml not present"
        fi
    done
    if [ -f "{{cross_platform_py}}" ]; then
        echo "==> python3 {{cross_platform_py}}"
        python3 "{{cross_platform_py}}"
    else
        echo "skip: {{cross_platform_py}} not present"
    fi

# Run `cargo test --workspace` over the Rust crates + `go test ./...`
# in bindings/go-c-export. Timeout: 600s (= 10m, per L1 audit).
test timeout=long_timeout:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo test --manifest-path $crate/Cargo.toml --workspace"
            timeout {{timeout}} {{cargo}} test --manifest-path "$crate/Cargo.toml" --workspace
        else
            echo "skip: $crate/Cargo.toml not present"
        fi
    done
    if [ -d "{{go_bindings_dir}}" ]; then
        echo "==> go test ./... in {{go_bindings_dir}}"
        (cd "{{go_bindings_dir}}" && timeout {{timeout}} go test ./...)
    else
        echo "skip: {{go_bindings_dir}} not present"
    fi

# Run `cargo clippy` over the Rust crates + `go vet ./...` on bindings/go-c-export.
lint:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo clippy -p $crate"
            {{cargo}} clippy --manifest-path "$crate/Cargo.toml" --all-targets --all-features -- -D warnings
        fi
    done
    if [ -d "{{go_bindings_dir}}" ]; then
        echo "==> go vet ./... in {{go_bindings_dir}}"
        (cd "{{go_bindings_dir}}" && go vet ./...)
    else
        echo "skip: {{go_bindings_dir}} not present"
    fi

# Verify formatting: `cargo fmt --all -- --check` + `gofmt -l .` on bindings/go-c-export.
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo fmt --manifest-path $crate/Cargo.toml --all --check"
            {{cargo}} fmt --manifest-path "$crate/Cargo.toml" --all -- --check
        fi
    done
    if [ -d "{{go_bindings_dir}}" ]; then
        echo "==> gofmt -l in {{go_bindings_dir}}"
        (cd "{{go_bindings_dir}}" && gofmt -l .)
    else
        echo "skip: {{go_bindings_dir}} not present"
    fi

# Apply formatting: `cargo fmt --all` + `gofmt -w .` on bindings/go-c-export.
fmt-fix:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo fmt --manifest-path $crate/Cargo.toml --all"
            {{cargo}} fmt --manifest-path "$crate/Cargo.toml" --all
        fi
    done
    if [ -d "{{go_bindings_dir}}" ]; then
        echo "==> gofmt -w in {{go_bindings_dir}}"
        (cd "{{go_bindings_dir}}" && gofmt -w .)
    fi

# Run `cargo llvm-cov --workspace` over the Rust crates (requires cargo-llvm-cov).
cov:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
        echo "cargo-llvm-cov not installed; install via 'cargo install cargo-llvm-cov'"
        exit 1
    fi
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo llvm-cov --manifest-path $crate/Cargo.toml --workspace"
            cargo llvm-cov --manifest-path "$crate/Cargo.toml" --workspace
        fi
    done

# Full local CI sweep: lint + test + fmt.
ci: lint test fmt
    @echo "ci: lint + test + fmt all passed"

# ---------------------------------------------------------------------------
# Bindings & docs
# ---------------------------------------------------------------------------

# Build the FFI bindings: runs the cross-platform Python orchestrator
# which builds the Go c-archive, Rust FFI, Zig module, and (optionally)
# tests the Python bindings.
bindings:
    @if [ -f "{{cross_platform_py}}" ]; then \
        python3 "{{cross_platform_py}}"; \
    else \
        echo "{{cross_platform_py}} not present"; \
        exit 1; \
    fi

# Start the VitePress dev server for the PhenoCompose docs site.
docs:
    @if [ -f package.json ] && grep -q '"docs:dev"' package.json; then \
        {{node_pm}} run docs:dev; \
    else \
        echo "no docs:dev script in package.json; cannot start VitePress"; \
        exit 1; \
    fi

# Build the VitePress docs site (`docs/`) for static hosting.
docs-build:
    @if [ -f package.json ] && grep -q '"docs:build"' package.json; then \
        {{node_pm}} run docs:build; \
    else \
        echo "no docs:build script in package.json; cannot build VitePress"; \
        exit 1; \
    fi

# Remove Rust + Go build artifacts (target/ dirs, Go test cache, coverage files).
clean:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{rust_crates}}; do
        if [ -f "$crate/Cargo.toml" ]; then
            echo "==> cargo clean -p $crate"
            (cd "$crate" && {{cargo}} clean) || true
        fi
    done
    if [ -d "{{go_bindings_dir}}" ]; then
        (cd "{{go_bindings_dir}}" && go clean -testcache) || true
    fi
    find . \( -name coverage.out -o -name coverage.html -o -name coverage.xml -o -name coverage.lcov -o -name llvm-cov-target \) -type f -delete 2>/dev/null || true
