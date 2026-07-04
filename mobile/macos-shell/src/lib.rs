//! PhenoCompose macOS shell-extension scaffold.
//!
//! Provides the Rust-side surface that `pheno-compose-driver` exposes to a
//! macOS shell extension / menu-bar app built with `swift-rs`. The actual
//! Swift side lives in `mobile/macos-shell/swift/` and is wired in by the
//! `swift` feature (target-gated).
//!
//! PILLAR-TAXONOMY-v2 cross-references: **L121** (macOS Native FFI — AppKit
//! via swift-rs) and **L130** (System Service Integration — launchd plist
//! host for service-mode operation of `pheno-compose-driver`).
//!
//! Status: scaffold only. The Swift sources, the `Info.plist`, and the
//! launchd plist are placeholders — full AppKit / Keychain / Spotlight
//! wiring lands in a follow-up PR once the FFI shape is validated.

#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

pub mod launchd;

#[cfg(target_os = "macos")]
#[cfg(feature = "swift")]
pub mod ffi;

#[derive(Debug, Error)]
pub enum MacosShellError {
    #[error("launchd bootstrap failed: {0}")]
    LaunchdBootstrap(String),

    #[error("Swift↔Rust bridge unavailable on this target")]
    BridgeUnavailable,

    #[error("shell extension returned non-zero status: {0}")]
    ExtensionFailed(i32),
}

/// High-level capability flags reported back to the host driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellCapabilities {
    pub appkit_bridge: bool,
    pub keychain: bool,
    pub launchd_service: bool,
    pub spotlight: bool,
}

impl Default for ShellCapabilities {
    fn default() -> Self {
        Self {
            appkit_bridge: cfg!(target_os = "macos"),
            keychain: cfg!(target_os = "macos"),
            launchd_service: cfg!(target_os = "macos"),
            spotlight: cfg!(target_os = "macos"),
        }
    }
}

/// Boot the shell extension. On non-macOS targets (or when the `swift`
/// feature is disabled) this returns the stub capability set so the host
/// driver can still smoke-test the call path.
pub fn boot_shell_extension() -> Result<ShellCapabilities, MacosShellError> {
    #[cfg(all(target_os = "macos", feature = "swift"))]
    {
        ffi::pheno_shell_boot().map_err(MacosShellError::ExtensionFailed)?;
    }
    #[cfg(not(all(target_os = "macos", feature = "swift")))]
    {
        // Stub path — used during scaffolding and on cross-compile smoke
        // checks (`cargo check --workspace` on linux).
    }
    Ok(ShellCapabilities::default())
}

/// Forward a deploy-event from `pheno-compose-driver` to the shell
/// extension. Returns `Ok(())` on all targets; non-macOS builds are no-ops.
pub fn notify_deploy_event(event_id: &str, payload: &str) -> Result<(), MacosShellError> {
    #[cfg(all(target_os = "macos", feature = "swift"))]
    {
        ffi::pheno_shell_notify(event_id, payload).map_err(MacosShellError::ExtensionFailed)?;
    }
    let _ = (event_id, payload);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_returns_default_capabilities_on_stub_path() {
        let caps = boot_shell_extension().expect("stub boot must succeed");
        assert_eq!(caps.appkit_bridge, cfg!(target_os = "macos"));
    }

    #[test]
    fn notify_is_noop_on_stub_path() {
        notify_deploy_event("evt-001", "{\"deploy_id\":\"d-1\"}")
            .expect("stub notify must succeed");
    }
}