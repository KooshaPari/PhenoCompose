> **Work state:** EPIC_A · **DAG phase:** A9 · **Progress:** `█████░░░░░ 50%`
> README freshness + work-state header standardization for PhenoCompose — part of epic_A hygiene sweep (compute/infra). · updated 2026-06-29
![Rust CI](https://github.com/KooshaPari/PhenoCompose/actions/workflows/rust-ci.yml/badge.svg?branch=main)

PhenoCompose is the Phenotype project’s Compose-facing orchestration layer for unified container and microVM workflows, pairing a CLI-driven developer experience with the project’s isolation and runtime abstractions. This repository README now front-loads the current work state, a clear progress indicator, and the minimum usage path so a reader can orient quickly before diving into the fuller project history below.

## Usage / Quickstart

```bash
git clone https://github.com/KooshaPari/PhenoCompose.git
cd PhenoCompose
go build ./...
go test ./...
```

Use `go build` to confirm the repo compiles, then `go test` to verify the current codebase. For active development, follow the repo guidance in `CLAUDE.md` before making changes.

<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->

![Downloads](https://img.shields.io/github/downloads/KooshaPari/PhenoCompose/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/PhenoCompose?style=flat-square&label=release)
![License](https://img.shields.io/github/license/KooshaPari/PhenoCompose?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**

> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.

<!-- AI-DD-META:END -->

## Work State

| Field       | Value                              |
| ----------- | ---------------------------------- |
| Last commit | 2026-06-07                         |
| Open issues | 0                                  |
| Open PRs    | 1                                  |
| Focus       | workflow hygiene + module path fix |

Progress: █████░░░░░ 50%

# NVMS - NanoVM Service (Unified)

[![Build](https://img.shields.io/github/actions/workflow/status/KooshaPari/PhenoCompose/rust-ci.yml?branch=main&label=build)](https://github.com/KooshaPari/PhenoCompose/actions/workflows/rust-ci.yml)
[![Release](https://img.shields.io/github/v/release/KooshaPari/PhenoCompose?include_prereleases&sort=semver)](https://github.com/KooshaPari/PhenoCompose/releases)
[![License](https://img.shields.io/github/license/KooshaPari/PhenoCompose)](LICENSE)
[![Phenotype](https://img.shields.io/badge/Phenotype-org-blueviolet)](https://github.com/KooshaPari)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/KooshaPari/PhenoCompose/actions/workflows/ci.yml/badge.svg)](https://github.com/KooshaPari/PhenoCompose/actions/workflows/ci.yml)
[![Go](https://img.shields.io/badge/go-1.21%2B-00ADD8.svg)](https://go.dev)

> **Merged Implementation**: KooshaPari/nanovms + BytePort/nvms + PhenoCompose Driver

> **Consolidated**: As of 2026-06-14, all polyglot bindings (Rust, Go, Mojo, Zig) have been migrated. See thegent/nvms and nanovms/sdk/rust for the canonical implementations.

NVMS provides **3-tier isolation** for secure, efficient application deployment:

- **Tier 1 (WASM)**: ~1ms startup, fast tools, trusted code
- **Tier 2 (gVisor)**: ~90ms startup, browser automation, semi-trusted
- **Tier 3 (Firecracker)**: ~125ms startup, full isolation, untrusted code

## Migration Status

| Component         | New Home                                   | Status               |
| ----------------- | ------------------------------------------ | -------------------- |
| Rust FFI + driver | `thegent/crates/thegent-nvms`              | Migrated             |
| Go C-export       | `nanovms/cmd/nvms-cgo`                     | Migrated             |
| Python bindings   | `thegent/crates/thegent-nvms` (pyo3)       | Migrated             |
| Mojo bindings     | `thegent/src/thegent/infra/mojo_bridge.py` | Replaced by bridge   |
| Zig bindings      | `thegent/crates/thegent-wasm-tools`        | Replaced by Wasm SDK |

## Quick Start (Legacy)

> **Note**: The unified interface is now `thegent` or `nvms-sdk` (Rust).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    UNIFIED NVMS STACK                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │ PhenoCompose│    │   NVMS CLI  │    │  BytePort   │    │
│  │   (Rust)    │    │    (Go)     │    │   (Go)      │    │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    │
│         │                  │                  │            │
│         └──────────────────┴──────────────────┘            │
│                            │                                │
│                    ┌───────▼───────┐                        │
│                    │   NVMS Core   │                        │
│                    │    (Merged)   │                        │
│                    └───────┬───────┘                        │
│                            │                                │
│         ┌──────────────────┼──────────────────┐            │
│         ▼                  ▼                  ▼            │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐        │
│  │    WASM    │    │   gVisor   │    │ Firecracker│        │
│  │  (~1ms)    │    │  (~90ms)   │    │  (~125ms)  │        │
│  └────────────┘    └────────────┘    └────────────┘        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Merge History

| Component                 | Source             | Status     | Contribution                 |
| ------------------------- | ------------------ | ---------- | ---------------------------- |
| **Core 3-tier isolation** | KooshaPari/nanovms | ✅ Complete | WASM/gVisor/Firecracker      |
| **AWS deployment**        | BytePort/nvms      | ✅ Merged   | Firecracker orchestration    |
| **Unified interface**     | PhenoCompose       | ✅ New      | Rust driver, standardization |

## Key Features

- **Multi-Tier Isolation Strategy** — Choose isolation level (WASM, gVisor, Firecracker) based on trust model and performance requirements
- **Unified Interface** — Single `pheno-compose` CLI for all isolation backends; configuration portable across tiers
- **Sub-Second Cold Starts** — WASM tier enables 1ms startup for rapid scaling and function-as-a-service workloads
- **Container Compatibility** — gVisor tier runs standard OCI containers without hardware virtualization
- **Full Virtualization** — Firecracker tier provides complete OS-level isolation for untrusted or legacy code
- **Resource Metering** — Track CPU, memory, I/O per workload with automatic enforcement
- **Networking** — Bridge or overlay network modes; DNS resolution via Phenotype service mesh
- **Volume Management** — Persistent volumes, ephemeral scratch, read-only root filesystem support
- **Observability** — Built-in logging, metrics (Prometheus), distributed tracing (Tempo integration)

## Platform Support

| Platform    | Tier 1 (WASM) | Tier 2 (gVisor) | Tier 3 (Firecracker)       |
| ----------- | ------------- | --------------- | -------------------------- |
| **macOS**   | ✅ Native      | ✅ Lima/VZ       | ✅ Virtualization.framework |
| **Linux**   | ✅ Native      | ✅ Native        | ✅ KVM                      |
| **Windows** | ✅ Native      | ✅ WSL2          | ✅ WSL2                     |

## Installation

```bash
# Install NVMS
curl -fsSL https://get.nvms.dev | sh

# Or build from source
git clone https://github.com/KooshaPari/nvms.git
cd nvms && go build ./cmd/nvms

# Install PhenoCompose driver
cargo install pheno-compose --features nvms-driver
```

## Features

- **Multi-Tier Isolation** — WASM, gVisor, Firecracker for different trust/performance tradeoffs
- **Unified Orchestration** — PhenoCompose driver standardizes deployment across tiers
- **Cross-Platform** — Native support for macOS, Linux, Windows
- **Fast Startup** — WASM in milliseconds for dev/testing workloads
- **Secure Isolation** — gVisor/Firecracker for untrusted code execution

## Project Status

- **Status**: Active
- **Languages**: Go (core) + Rust (PhenoCompose driver)
- **Type**: Container/Sandbox Orchestration
- **Part of**: Phenotype Ecosystem
- **Integrates With**: BytePort, nanovms, AgilePlus

## Quality & Testing

- Functional requirements tracked in AgilePlus
- Platform compatibility tests for each tier
- Integration tests with Firecracker and gVisor
- Deployment verification across cloud platforms

## Documentation

- [PhenoCompose Integration](integrations/pheno-compose/README.md)
- [AWS Deployment](docs/aws-deployment.md)
- [Architecture Guide](docs/architecture.md)
- **Worklogs**: Audit trail in `docs/worklogs/` (if present)
- **Governance**: See `CLAUDE.md` for development rules

## License

Apache-2.0

## License

MIT — see [LICENSE](./LICENSE).
