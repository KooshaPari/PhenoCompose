# DESIGN.md — PhenoCompose

## Overview

**PhenoCompose** is a Rust-based process manager inspired by process-compose, with added NVMS (NanoVMs) support. It provides declarative process orchestration, health checking, and lifecycle management for local and cloud workloads.

## Architecture

```
PhenoCompose/
├── crates/                # Rust crates (core, driver, NVMS integration)
│   ├── pheno-compose/     # Core process orchestration engine
│   └── pheno-compose-driver/ # External driver interface
├── pheno-compose-driver/  # Driver binary (Svelte UI + Rust backend)
├── bindings/              # Language bindings (Node.js, Python)
├── packages/              # Shared TypeScript packages
├── integrations/          # Third-party integration adapters
├── ports/                 # Port allocation and mapping
├── internal/              # Internal utility modules
├── examples/              # Example compose files
├── audits/                # Quality audit reports
└── tests/                 # Integration and E2E tests
```

## Key Design Decisions

1. **Rust core with bindings** — process management in Rust, exposed to Node.js/Python via FFI
2. **NVMS-native support** — first-class integration with NanoVMs for unikernel process deployment
3. **Declarative compose files** — YAML-based process definitions with health checks and dependencies
4. **Driver architecture** — pluggable backends for local, Docker, and NVMS execution environments

## Data Flow

```
Compose File (YAML) → Parser → Process Graph Builder → Scheduler → Driver (Local/Docker/NVMS) → Health Monitor
```

## Non-Goals

- Docker Compose replacement (complements, not replaces)
- Kubernetes manifest generation (out of scope)
- Full observability stack (delegates to phenotype-infra)

## Status

- B39-grade release with multi-platform test coverage
- NVMS integration verified in integration tests
- Bindings for Node.js and Python published

## References

- [AGENTS.md](./AGENTS.md) — LLM contributor guidelines
- [SPEC.md](./SPEC.md) — Compose file specification
- [TEST_COVERAGE_MATRIX.md](./TEST_COVERAGE_MATRIX.md) — Coverage matrix
