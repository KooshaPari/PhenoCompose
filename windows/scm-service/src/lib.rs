//! PhenoCompose Windows SCM service-mode scaffold.
//!
//! Wires `pheno-compose-driver` into the Windows Service Control Manager
//! (SCM) so it can run as a registered Windows service (`sc start
//! PhenoCompose`). PILLAR-TAXONOMY-v2 **L123** (Windows Native FFI —
//! `microsoft/windows-rs`) and **L130** (System Service Integration —
//! Windows-service / SCM).
//!
//! Status: scaffold only. The actual SCM registration helpers land in a
//! follow-up PR once the FFI shape is validated. On non-Windows targets
//! the module exposes the stub surface so `cargo check --workspace`
//! continues to pass on linux.

#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

pub mod scm;

#[cfg(target_os = "windows")]
#[cfg(feature = "service")]
pub mod service_main;

pub const SERVICE_NAME: &str = "PhenoCompose";
pub const SERVICE_DISPLAY: &str = "PhenoCompose Driver (NVMS orchestrator)";
pub const SERVICE_DESCRIPTION: &str =
    "PhenoCompose NVMS driver — composition + 3-tier isolation. See PLAN.md.";

#[derive(Debug, Error)]
pub enum ScmServiceError {
    #[error("Windows SCM bridge unavailable on this target")]
    BridgeUnavailable,

    #[error("service registration failed: {0}")]
    RegistrationFailed(String),

    #[error("service start failed: {0}")]
    StartFailed(String),

    #[error("service stop failed: {0}")]
    StopFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScmCapabilities {
    pub scm_registration: bool,
    pub service_control: bool,
    pub event_log: bool,
}

impl Default for ScmCapabilities {
    fn default() -> Self {
        Self {
            scm_registration: cfg!(target_os = "windows"),
            service_control: cfg!(target_os = "windows"),
            event_log: cfg!(target_os = "windows"),
        }
    }
}

/// Query the platform-level capability set — useful for the host driver
/// to decide whether to attempt SCM bootstrap or skip with a friendly
/// message.
pub fn capabilities() -> ScmCapabilities {
    ScmCapabilities::default()
}

/// Install the service binary with the SCM. The Windows-only path uses
/// `CreateServiceW` from `Win32_System_Services`; on non-Windows the
/// call returns `BridgeUnavailable` so the scaffold compiles.
pub fn install_service(binary_path: &str) -> Result<(), ScmServiceError> {
    #[cfg(all(target_os = "windows", feature = "service"))]
    {
        service_main::install_via_scm(binary_path)?;
    }
    #[cfg(not(all(target_os = "windows", feature = "service")))]
    {
        let _ = binary_path;
        Err(ScmServiceError::BridgeUnavailable)
    }
    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        let _ = binary_path;
        Ok(())
    }
}

/// Send a `SERVICE_CONTROL_STOP` to the SCM-registered service.
pub fn stop_service() -> Result<(), ScmServiceError> {
    #[cfg(all(target_os = "windows", feature = "service"))]
    {
        service_main::stop_via_scm()
    }
    #[cfg(not(all(target_os = "windows", feature = "service")))]
    {
        Err(ScmServiceError::BridgeUnavailable)
    }
    #[cfg(all(target_os = "windows", not(feature = "service")))]
    {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_default_reflects_target() {
        let caps = capabilities();
        assert_eq!(caps.scm_registration, cfg!(target_os = "windows"));
    }

    #[test]
    fn install_off_windows_is_unsupported() {
        #[cfg(not(target_os = "windows"))]
        {
            let err = install_service("C:/pheno-compose-driver.exe").unwrap_err();
            assert!(matches!(err, ScmServiceError::BridgeUnavailable));
        }
    }

    #[test]
    fn stop_off_windows_is_unsupported() {
        #[cfg(not(target_os = "windows"))]
        {
            let err = stop_service().unwrap_err();
            assert!(matches!(err, ScmServiceError::BridgeUnavailable));
        }
    }
}