# Subsystems — PhenoCompose

ADR-038 cross-link: see [ADR-038: Hexagonal port-adapter L4 policy](https://github.com/KooshaPari/phenotype-apps/blob/main/docs/adr/2026-06-18/ADR-038-hexagonal-port-adapter-l4-policy.md) for the canonical input/output port contract.

> L7 subsystem decomposition. Bounded contexts, ports, owned data, external
> dependencies, and failure modes for the PhenoCompose multi-crate
> workspace (Compose-file-style orchestration for agents, drivers, and
> services). Companion to `ARCHITECTURE.md`. Initial decomposition
> 2026-06-21 (v16 cycle-6 T1).

## Subsystem map

| Subsystem | Path | Responsibility | Owned data | Critical? |
|---|---|---|---|---|
| Compose core | `crates/pheno-compose`, `src/` | Top-level compose file parser, DAG build, lifecycle orchestration | compose graph, lifecycle state | yes |
| Driver | `pheno-compose-driver/` | Driver process / runtime that executes a compose instance | instance state, driver socket | yes |
| Ports (L4) | `ports/` | Trait definitions: `Driver`, `Adapter`, `HealthCheck`, `Notifier` (per ADR-038) | none (interface only) | yes |
| Bindings | `bindings/rust-ffi`, `bindings/go-c-export`, `bindings/mojo`, `bindings/zig` | Cross-language FFI bindings (Rust-FFI, Go cgo, Mojo, Zig) | binding handle table | no |
| Integrations | `integrations/` | First-party integrations (e.g. K8s, Nomad, Docker, local) | integration adapters | no |
| Internal | `internal/` | Internal helpers (logging, retry, error mapping) | retry queue | no |

## Port catalogue

### Input ports (consumed)

- `pheno-config::Config` (via `Configra`) — layered config.
- `pheno-errors::Error` envelope.
- `pheno-tracing` OTLP exporter.
- `pheno-port-adapter` L4 port trait surface (per ADR-038).
- OS: Docker Engine API (over Unix socket), K8s API (over HTTPS) — adapter-specific.

### Output ports (produced)

- `pheno-compose-driver::Instance` — long-running driver handle.
- `ports::Driver`, `ports::Adapter`, `ports::HealthCheck`, `ports::Notifier` — public traits.
- Compose file (YAML/JSON) — public input format.
- Telemetry events on every compose event (via `pheno-tracing`).

## External dependencies

| Dependency | Kind | Used by |
|---|---|---|
| `pheno-config` | Cargo path (workspace) | config cascade |
| `pheno-errors` | Cargo path | error envelope |
| `pheno-tracing` | Cargo path | OTLP spans |
| `pheno-port-adapter` | Cargo path | L4 port trait surface |
| `serde_yaml`, `serde_json` | crates.io | compose file parse |
| `tokio` | crates.io | async runtime |
| `bollard` | crates.io | Docker client |
| `kube` | crates.io | K8s client |
| `cgo` (C) | system | Go bindings |
| `mojo` | system (optional) | Mojo bindings |
| `zig` | system (optional) | Zig bindings |

## Failure modes

| Subsystem | Failure | Detection | Recovery |
|---|---|---|---|
| Compose core | YAML parse error | `serde_yaml` error | report line/column; exit 1 |
| Compose core | DAG cycle | topological-sort fail | report cycle path; exit 1 |
| Compose core | lifecycle timeout | `tokio::time::timeout` | emit `LifecycleTimeout`; cleanup |
| Driver | driver crash | driver socket close | respawn; replay last good state |
| Driver | instance drift | hash compare vs spec | reconcile; emit `DriftDetected` |
| Ports (L4) | trait mismatch across versions | compile-time | Cargo workspace version pin |
| Bindings | FFI handle leak | handle table audit | free on consumer close |
| Bindings | cgo null pointer | Go nil-check | panic → exit; surface binding error |
| Integrations (Docker) | daemon unreachable | bollard connection error | retry with backoff; max 3 |
| Integrations (K8s) | RBAC denied | 403 from API | surface `K8sDenied`; abort |
| Integrations (K8s) | CRD missing | 404 from API | surface `CRDMissing`; abort |

## Change log

- 2026-06-21 — initial decomposition (v16 cycle-6 T1, L7). 6 subsystems (5 + internal helpers). ADR-038 cross-link added.
