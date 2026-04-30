# NVMS - NanoVM Service (Unified)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Quality Gate](https://github.com/KooshaPari/PhenoCompose/actions/workflows/quality-gate.yml/badge.svg)](https://github.com/KooshaPari/PhenoCompose/actions/workflows/quality-gate.yml)
[![Go](https://img.shields.io/badge/go-1.21%2B-00ADD8.svg)](https://go.dev)
[![AI Slop Inside](https://sladge.net/badge.svg)](https://sladge.net)

> **Merged Implementation**: KooshaPari/nanovms + BytePort/nvms + PhenoCompose Driver

NVMS provides **3-tier isolation** for secure, efficient application deployment:
- **Tier 1 (WASM)**: ~1ms startup, fast tools, trusted code
- **Tier 2 (gVisor)**: ~90ms startup, browser automation, semi-trusted
- **Tier 3 (Firecracker)**: ~125ms startup, full isolation, untrusted code

## Quick Start

```bash
# Deploy with NVMS
nvms deploy --tier 1 --config nvms.yaml  # WASM
nvms deploy --tier 2 --config nvms.yaml  # gVisor
nvms deploy --tier 3 --config nvms.yaml  # Firecracker

# Or use PhenoCompose (unified interface)
pheno-compose deploy --runtime nvms --config nvms.yaml
```

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

| Component | Source | Status | Contribution |
|-----------|--------|--------|--------------|
| **Core 3-tier isolation** | KooshaPari/nanovms | ✅ Complete | WASM/gVisor/Firecracker |
| **AWS deployment** | BytePort/nvms | ✅ Merged | Firecracker orchestration |
| **Unified interface** | PhenoCompose | ✅ New | Rust driver, standardization |

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

| Platform | Tier 1 (WASM) | Tier 2 (gVisor) | Tier 3 (Firecracker) |
|----------|---------------|-----------------|----------------------|
| **macOS** | ✅ Native | ✅ Lima/VZ | ✅ Virtualization.framework |
| **Linux** | ✅ Native | ✅ Native | ✅ KVM |
| **Windows** | ✅ Native | ✅ WSL2 | ✅ WSL2 |

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
