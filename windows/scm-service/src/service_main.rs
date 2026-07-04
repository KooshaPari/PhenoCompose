//! Windows-only ServiceMain bridge. Compiled in only when both
//! `target_os = "windows"` AND the `service` feature are enabled.
//!
//! Real implementation will:
//!   1. Provide an `extern "system" fn ServiceMain(_: u32, _: *mut *mut u16)`
//!      that calls `RegisterServiceCtrlHandlerExW`.
//!   2. Set up a `SERVICE_STATUS_HANDLE` and report `SERVICE_RUNNING`.
//!   3. Spawn the `pheno-compose-driver` async runtime and forward
//!      `SERVICE_CONTROL_STOP` / `SERVICE_CONTROL_SHUTDOWN` events.
//!
//! For now the module exposes only the platform-gated entry points so
//! the scaffold compiles cross-platform.
//!
//! PILLAR-TAXONOMY-v2 **L123** (Windows Native FFI).

#![cfg(all(target_os = "windows", feature = "service"))]

use super::ScmServiceError;

/// Stub for the SCM registration call. Real implementation will use
/// `windows::Win32::System::Services::{OpenSCManagerW, CreateServiceW}`.
pub fn install_via_scm(binary_path: &str) -> Result<(), ScmServiceError> {
    let _ = binary_path;
    Ok(())
}

/// Stub for the SCM `ControlService(SERVICE_CONTROL_STOP)` call.
pub fn stop_via_scm() -> Result<(), ScmServiceError> {
    Ok(())
}