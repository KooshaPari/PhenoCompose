// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types for the PhenoCompose NVMS driver.
//!
//! [`DriverError`] wraps FFI-level [`NvmsError`](nvms_ffi::NvmsError)
//! with operation context and a human-readable recovery hint so
//! callers can respond to failures without inspecting raw FFI
//! error codes.

use nvms_ffi::NvmsError;
use std::fmt;

use crate::Tier;

/// Errors originating from the PhenoCompose NVMS driver.
///
/// Each variant carries contextual information about the operation
/// that failed.  Use [`DriverError::recovery_hint`] to get a
/// human-readable suggestion for resolving the failure.
#[derive(Debug)]
pub enum DriverError {
    /// NVMS library initialization failed.
    Init {
        /// The underlying FFI error.
        source: NvmsError,
    },
    /// Instance creation failed.
    ///
    /// Typical causes: an instance name containing a null byte, or
    /// the NVMS backend cannot allocate a new instance.
    CreateInstance {
        /// The tier that was requested.
        tier: Tier,
        /// The instance name that was passed.
        name: String,
        /// The underlying FFI error.
        source: NvmsError,
    },
    /// Instance start failed.
    Start {
        /// The instance ID (best-effort — may be `0` when the
        /// instance handle is still being set up).
        instance_id: u64,
        /// The underlying FFI error.
        source: NvmsError,
    },
    /// Instance stop failed.
    Stop {
        /// The instance ID.
        instance_id: u64,
        /// The underlying FFI error.
        source: NvmsError,
    },
    /// Instance destroy (Drop) failed.
    Destroy {
        /// The instance ID.
        instance_id: u64,
    },
    /// Configuration error (invalid or conflicting config values).
    Config(String),
}

impl DriverError {
    /// Return a human-readable recovery hint for this error.
    ///
    /// Hints are static strings intended for log messages or
    /// user-facing error displays.  They explain *what the
    /// caller should do next*, not what went wrong internally.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::Init { .. } => {
                "Ensure the NVMS shared library is installed and on \
                 LD_LIBRARY_PATH / DYLD_LIBRARY_PATH, then retry."
            }
            Self::CreateInstance { name, .. } => {
                if name.contains('\0') {
                    "Instance name must not contain null bytes. \
                     Remove any embedded NUL characters and retry."
                } else {
                    "Check that the NVMS backend has capacity to \
                     create a new instance (memory, CPU slots)."
                }
            }
            Self::Start { .. } => {
                "Ensure the instance is in a valid state to be \
                 started (it must not already be running or have \
                 been destroyed)."
            }
            Self::Stop { .. } => {
                "Ensure the instance is running and has not been \
                 destroyed.  Stopping a stopped instance is \
                 idempotent at the driver level."
            }
            Self::Destroy { .. } => {
                "Instance resources may have leaked.  Check the \
                 NVMS backend for orphaned instances."
            }
            Self::Config(_) => {
                "Review the configuration values passed to the \
                 driver or loaded from config files."
            }
        }
    }

    /// Returns the underlying [`NvmsError`] code, if applicable.
    pub fn raw_code(&self) -> Option<i32> {
        match self {
            Self::Init { source } => Some(source.raw_code()),
            Self::CreateInstance { source, .. } => Some(source.raw_code()),
            Self::Start { source, .. } => Some(source.raw_code()),
            Self::Stop { source, .. } => Some(source.raw_code()),
            Self::Destroy { .. } => None,
            Self::Config(_) => None,
        }
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init { source } => {
                write!(f, "NVMS driver init failed: {source}")
            }
            Self::CreateInstance { tier, name, source } => {
                write!(f, "failed to create {tier:?} instance \"{name}\": {source}")
            }
            Self::Start {
                instance_id, source, ..
            } => {
                write!(f, "failed to start instance {instance_id}: {source}")
            }
            Self::Stop {
                instance_id, source, ..
            } => {
                write!(f, "failed to stop instance {instance_id}: {source}")
            }
            Self::Destroy { instance_id } => {
                write!(f, "failed to destroy instance {instance_id}: resources may have leaked")
            }
            Self::Config(msg) => write!(f, "NVMS driver config error: {msg}"),
        }
    }
}

impl std::error::Error for DriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Init { source } => Some(source),
            Self::CreateInstance { source, .. } => Some(source),
            Self::Start { source, .. } => Some(source),
            Self::Stop { source, .. } => Some(source),
            Self::Destroy { .. } => None,
            Self::Config(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn driver_error_recovery_hints_are_not_empty() {
        let cases: Vec<DriverError> = vec![
            DriverError::Init {
                source: NvmsError::InitFailed,
            },
            DriverError::CreateInstance {
                tier: Tier::Wasm,
                name: "svc".into(),
                source: NvmsError::CreateFailed,
            },
            DriverError::CreateInstance {
                tier: Tier::Wasm,
                name: "bad\0name".into(),
                source: NvmsError::CreateFailed,
            },
            DriverError::Start {
                instance_id: 1,
                source: NvmsError::StartFailed,
            },
            DriverError::Stop {
                instance_id: 1,
                source: NvmsError::StopFailed,
            },
            DriverError::Destroy { instance_id: 1 },
            DriverError::Config("bad value".into()),
        ];
        for err in &cases {
            assert!(
                !err.recovery_hint().is_empty(),
                "recovery_hint() should not be empty for {err:?}"
            );
        }
    }

    #[test]
    fn driver_error_raw_code_returns_some_for_ffi_variants() {
        let err = DriverError::Init {
            source: NvmsError::InitFailed,
        };
        assert_eq!(err.raw_code(), Some(-1));
    }

    #[test]
    fn driver_error_raw_code_returns_none_for_config() {
        let err = DriverError::Config("bad value".into());
        assert_eq!(err.raw_code(), None);
    }

    #[test]
    fn driver_error_raw_code_returns_none_for_destroy() {
        let err = DriverError::Destroy { instance_id: 42 };
        assert_eq!(err.raw_code(), None);
    }

    #[test]
    fn driver_error_display_contains_context() {
        let err = DriverError::Init {
            source: NvmsError::InitFailed,
        };
        let msg = err.to_string();
        assert!(msg.contains("init"));
        assert!(msg.contains("NVMS"));

        let err = DriverError::CreateInstance {
            tier: Tier::Wasm,
            name: "test-svc".into(),
            source: NvmsError::CreateFailed,
        };
        let msg = err.to_string();
        assert!(msg.contains("test-svc"));
        assert!(msg.contains("Wasm"));
    }

    #[test]
    fn driver_error_error_source_returns_some_for_ffi() {
        let err = DriverError::Init {
            source: NvmsError::InitFailed,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn driver_error_error_source_returns_none_for_config() {
        let err = DriverError::Config("bad".into());
        assert!(err.source().is_none());
    }

    #[test]
    fn driver_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<DriverError>();
        assert_sync::<DriverError>();
    }
}
