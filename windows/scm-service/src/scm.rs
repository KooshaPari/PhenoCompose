//! Cross-platform surface that decides whether the SCM bridge is wired
//! in. On Windows + `service` feature this module re-exports the real
//! helpers; on every other build it returns `BridgeUnavailable` stubs.
//!
//! PILLAR-TAXONOMY-v2 **L123** (Windows Native FFI — `microsoft/windows-rs`)
//! and **L130** (System Service Integration — Windows-service / SCM).

/// Service start type registered with the SCM. Mirrors the
/// `SERVICE_AUTO_START` constant from `Win32_System_Services`.
pub const SERVICE_AUTO_START: u32 = 0x00000002;

/// Service type: `SERVICE_WIN32_OWN_PROCESS`.
pub const SERVICE_WIN32_OWN_PROCESS: u32 = 0x00000010;

/// Error control severity: `SERVICE_ERROR_NORMAL`.
pub const SERVICE_ERROR_NORMAL: u32 = 0x00000001;

/// Compose the full service command line for `CreateServiceW`.
/// Real implementation will call `sc.exe create` or
/// `OpenSCManagerW` + `CreateServiceW` — the constants above mirror the
/// native Win32 values so the JSON manifest stays stable cross-platform.
pub fn compose_create_command(binary_path: &str) -> String {
    format!(
        "sc create \"{name}\" binPath= \"{bin}\" start= auto DisplayName= \"{disp}\"",
        name = super::SERVICE_NAME,
        bin = binary_path,
        disp = super::SERVICE_DISPLAY,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_command_contains_service_metadata() {
        let cmd = compose_create_command(r"C:\Program Files\PhenoCompose\pheno-compose-driver.exe");
        assert!(cmd.contains(super::super::SERVICE_NAME));
        assert!(cmd.contains("DisplayName"));
        assert!(cmd.contains("start= auto"));
        assert!(cmd.contains(r"C:\Program Files\PhenoCompose\pheno-compose-driver.exe"));
    }
}