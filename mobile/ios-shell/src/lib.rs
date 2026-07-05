//! pheno-ios-shell: iOS shell-extension + deploy-monitor for PhenoCompose.
//!
//! L122: iOS Native FFI. Mirrors the L121 (macos-shell) and L125 (android-monitor) pattern.
//! Default features empty; build with `--features native-bridge` to enable the Swift bridge.

#![cfg_attr(feature = "native-bridge", allow(unsafe_code))]

pub const IOS_FFI_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployState {
    Idle,
    Deploying,
    Running,
    Failed,
}

impl DeployState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeployState::Idle => "idle",
            DeployState::Deploying => "deploying",
            DeployState::Running => "running",
            DeployState::Failed => "failed",
        }
    }
}

pub fn ping() -> &'static str {
    "pheno-ios-shell:ok"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn v() { assert!(IOS_FFI_VERSION.starts_with("0.")); }
    #[test] fn p() { assert_eq!(ping(), "pheno-ios-shell:ok"); }
    #[test] fn s() { for s in [DeployState::Idle, DeployState::Deploying, DeployState::Running, DeployState::Failed] { assert!(!s.as_str().is_empty()); } }
}
