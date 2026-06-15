# /Users/kooshapari/CodeProjects/Phenotype/repos/PhenoCompose/justfile
# PhenoCompose dev workflow — post-consolidation

set shell := ["bash", "-uc"]
set dotenv-load

_default:
    @just --list

# ---------- Workspace ----------

# Type-check the entire workspace
check:
    cargo check --workspace --all-targets

# Build the workspace
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace --all-features

# Lint
lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all -- --check

# Format
fmt:
    cargo fmt --all

# ---------- Per-crate ----------

# Test a single port-trait crate (e.g., just test-one port-types)
test-one crate:
    cargo test -p phenocompose-port-{{crate}}

# Build a single port-trait crate
build-one crate:
    cargo build -p phenocompose-port-{{crate}}

# ---------- CI ----------

# Full CI: check + test + lint
ci: check test lint

# ---------- Security ----------

audit:
    cargo audit

deny:
    cargo deny check

# ---------- Docs ----------

doc:
    cargo doc --workspace --no-deps

doc-open:
    cargo doc --workspace --no-deps --open

# ---------- Coverage ----------

coverage:
    cargo llvm-cov --workspace --lcov --output-path lcov.info
    cargo llvm-cov report

# ---------- Cleanup ----------

clean:
    cargo clean
    rm -rf target

# ---------- Release ----------

release version:
    git cliff --tag {{version}} --output CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "chore(release): {{version}}"
    git tag -a {{version}} -m "Release {{version}}"
