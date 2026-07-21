# Windows SCM Service-Mode — PhenoCompose

PILLAR-TAXONOMY-v2 lines **L123** (Windows Native FFI — `microsoft/windows-rs`)
and **L130** (System Service Integration — Windows-service / SCM) tracked here.

PhenoCompose is a Rust orchestrator workspace; it currently exposes
`pheno-compose-driver` as a long-running daemon binary. PR-B adds the
Windows-side wiring that registers the driver with the Windows Service
Control Manager (SCM) so it starts on boot, restarts on crash, and is
manageable via `sc.exe` / PowerShell `Get-Service`.

## Layout

| Path | Crate | Target | Purpose |
| --- | --- | --- | --- |
| `windows/scm-service/` | `pheno-scm-service` | `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` | SCM registration + service control via `windows = "0.58"` |
| `windows/scm-service/service-assets/` | — | (shipped alongside the binary) | PowerShell installer/uninstaller, side-by-side manifest, event-log `.mc` |

## Feature flags

The crate is designed to **compile on linux-x86_64** even though the
Win32 bindings are Windows-only:

| Crate | Default features | Native feature |
| --- | --- | --- |
| `pheno-scm-service` | `[]` | `service` — pulls in `windows = "0.58"` with `Win32_System_Services` + `Win32_Foundation` |

The `service` feature is additionally gated to `target_os = "windows"`
in `[target.'cfg(target_os = "windows")'.dependencies]`. Result:
`cargo check --workspace --all-targets` continues to pass on linux
without a Windows toolchain.

## Cross-compile matrix

| Target | Status | Notes |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | ✅ scaffold passes `cargo check` | default CI target |
| `x86_64-pc-windows-msvc` | 🔄 needs MSVC + Windows SDK | enabled by `--features service` |
| `aarch64-pc-windows-msvc` | 🔄 needs MSVC + ARM64 SDK | enabled by `--features service` |

### Build matrix recipes

```bash
# Windows SCM service binary
cargo build --release --target x86_64-pc-windows-msvc \
    -p pheno-scm-service --features service
```

## Public API summary

| Item | Where |
| --- | --- |
| `capabilities() -> ScmCapabilities` | `src/lib.rs:67` |
| `install_service(binary_path) -> Result<(), ScmServiceError>` | `src/lib.rs:74` |
| `stop_service() -> Result<(), ScmServiceError>` | `src/lib.rs:91` |
| `scm::compose_create_command(binary_path) -> String` | `src/scm.rs:24` |
| `SERVICE_NAME = "PhenoCompose"` | `src/lib.rs:18` |
| `SERVICE_AUTO_START`, `SERVICE_WIN32_OWN_PROCESS`, `SERVICE_ERROR_NORMAL` | `src/scm.rs:9-15` |

## SCM registration — shape of the eventual implementation

`install_service()` will:

1. `OpenSCManagerW` (machine `NULL`, db `SERVICES_ACTIVE_DATABASE`,
   access `SC_MANAGER_ALL_ACCESS`).
2. `CreateServiceW` with the constants from `scm.rs`:
   `dwServiceType = SERVICE_WIN32_OWN_PROCESS`,
   `dwStartType = SERVICE_AUTO_START`,
   `dwErrorControl = SERVICE_ERROR_NORMAL`.
3. `CloseServiceHandle` on both handles.
4. Optionally invoke `ChangeServiceConfig2W(SERVICE_CONFIG_DESCRIPTION)`
   with `SERVICE_DESCRIPTION` from `src/lib.rs:20`.

`stop_service()` will call `ControlService(SERVICE_CONTROL_STOP)` then
poll `QueryServiceStatus` until `dwCurrentState == SERVICE_STOPPED`.

## Audit context

Source: `PhenoCompose-audit.json` (lines 16, 64, 230, 242, 296). Pre-PR
L123 score: 1/100 ("Rust compiles for windows-msvc but no Win32/WinRT
bindings"). PR-B introduces the minimum-viable `windows-rs` surface to
start raising that score; the real `ServiceMain` /
`RegisterServiceCtrlHandlerExW` plumbing lands in a follow-up PR.

## Out of scope (follow-up PRs)

- **Event-log channel** (`event-log.mc`) — needs `mc.exe` from the Windows SDK
- **Tray icon / WinUI 3** — PILLAR-TAXONOMY-v2 L129 (Native Notification Surfaces)
- **MSI / WiX installer** — distinct workstream, gated on `cargo-dist` selection
- **WinRT projection** — only needed if/when a packaged Windows app ships