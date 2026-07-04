# Windows SCM Service Assets (PILLAR-TAXONOMY-v2 L123/L130)

The Rust crate `pheno-scm-service` wires `pheno-compose-driver` into the
Windows Service Control Manager (SCM). This directory holds the
**non-code** assets that ship alongside the binary on Windows.

## Files (planned, follow-up PR)

| File | Role |
| --- | --- |
| `pheno-compose-driver.exe.manifest` | Side-by-side assembly manifest (DPI awareness, long-path awareness, requestedExecutionLevel=asInvoker) |
| `install.ps1` | PowerShell wrapper that calls `pheno-scm-service install --path <bin>` then `sc start PhenoCompose` |
| `uninstall.ps1` | `sc stop` + `sc delete PhenoCompose` |
| `event-log.mc` | Message Compiler source for the PhenoCompose event-log channel (Information / Warning / Error severities) |
| `README.md` | Operator runbook (this directory's index) |

## Install (operator runbook, post-merge)

```powershell
# 1. Build the driver + the SCM shim
cargo build --release --target x86_64-pc-windows-msvc -p pheno-compose-driver
cargo build --release --target x86_64-pc-windows-msvc -p pheno-scm-service --features service

# 2. Install
.\install.ps1 -BinaryPath "C:\Program Files\PhenoCompose\pheno-compose-driver.exe"

# 3. Verify
sc query PhenoCompose
Get-EventLog -LogName Application -Source PhenoCompose -Newest 10
```

## Uninstall

```powershell
.\uninstall.ps1
```

## Scaffold status

PILLAR-TAXONOMY-v2 **L130** (Windows-service / SCM). The Rust crate
(`pheno-scm-service`) already exposes `install_service()` and
`stop_service()`; the PowerShell wrappers will land alongside the
follow-up PR that introduces the real `Win32_System_Services` calls.