// SPDX-License-Identifier: MIT OR Apache-2.0
//! PhenoCompose NVMS Driver
//!
//! High-level Rust driver for NVMS integration with PhenoCompose.
//! This driver provides a safe, idiomatic Rust interface to the
//! NVMS Go library via FFI.
//!
//! # Architecture
//!
//! ```text
//! PhenoCompose (Rust)
//!     └── PhenoComposeDriver
//!             └── nvms_ffi (Rust FFI bindings)
//!                     └── NVMS Go Core (via CGO)
//! ```
//!
//! # Usage
//!
//! ```rust
//! use pheno_compose_driver::{NvmsDriver, Tier};
//!
//! # fn main() -> Result<(), pheno_compose_driver::nvms_ffi::NvmsError> {
//! let driver = NvmsDriver::new()?;
//! let mut instance = driver.create_instance(Tier::Wasm, "my-service")?;
//! instance.start()?;
//! # Ok(())
//! # }
//! ```

mod config;
mod instance;

pub use config::NvmsConfig;
pub use instance::{Instance, InstanceStatus, Tier};

pub use nvms_ffi;
use nvms_ffi::{NvmsError, Tier as FfiTier};

/// NVMS Driver for PhenoCompose
///
/// Provides high-level access to NVMS 3-tier isolation
pub struct NvmsDriver {
    version: String,
}

impl NvmsDriver {
    /// Create a new NVMS driver
    pub fn new() -> Result<Self, NvmsError> {
        nvms_ffi::init()?;
        Ok(Self {
            version: nvms_ffi::version(),
        })
    }

    /// Get NVMS version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Create a new instance with the specified tier
    pub fn create_instance(
        &self,
        tier: Tier,
        name: &str,
    ) -> Result<Instance, NvmsError> {
        let ffi_tier: FfiTier = tier.into();
        let c_name = std::ffi::CString::new(name).map_err(|_| NvmsError::CreateFailed)?;
        let ptr = unsafe { nvms_ffi::sys::nvms_instance_create(ffi_tier.into(), c_name.as_ptr()) };
        if ptr.is_null() {
            return Err(NvmsError::CreateFailed);
        }
        unsafe { Instance::from_ffi_ptr(ptr) }
    }

    /// Create instance with full configuration
    pub fn create_instance_with_config(
        &self,
        config: &NvmsConfig,
    ) -> Result<Instance, NvmsError> {
        let instance = self.create_instance(config.tier, &config.name)?;
        // Apply additional config options here
        Ok(instance)
    }

    /// List all running instances
    pub fn list_instances(&self) -> Vec<InstanceInfo> {
        // TODO: Implement via FFI
        Vec::new()
    }
}

/// Information about an instance
#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub id: u64,
    pub name: String,
    pub tier: Tier,
    pub status: InstanceStatus,
}

impl Default for NvmsDriver {
    fn default() -> Self {
        Self::new().expect("Failed to initialize NVMS driver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_initialization() {
        let driver = NvmsDriver::new();
        assert!(driver.is_ok());

        let driver = driver.unwrap();
        assert!(driver.version().starts_with("1.0"));
    }

    #[test]
    fn test_create_wasm_instance() {
        let driver = NvmsDriver::new().unwrap();
        let instance = driver.create_instance(Tier::Wasm, "test-wasm");

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.tier(), Tier::Wasm);
        assert_eq!(instance.status(), InstanceStatus::Running);
    }

    #[test]
    fn test_create_gvisor_instance() {
        let driver = NvmsDriver::new().unwrap();
        let instance = driver.create_instance(Tier::Gvisor, "test-gvisor");

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.tier(), Tier::Gvisor);
    }

    #[test]
    fn test_create_firecracker_instance() {
        let driver = NvmsDriver::new().unwrap();
        let instance = driver.create_instance(Tier::Firecracker, "test-fc");

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.tier(), Tier::Firecracker);
    }

    #[test]
    fn test_instance_lifecycle() {
        let driver = NvmsDriver::new().unwrap();
        let mut instance = driver.create_instance(Tier::Wasm, "lifecycle-test").unwrap();

        // Start
        assert!(instance.start().is_ok());
        assert_eq!(instance.status(), InstanceStatus::Running);

        // Stop
        assert!(instance.stop().is_ok());
        assert_eq!(instance.status(), InstanceStatus::Stopped);

        // Start again
        assert!(instance.start().is_ok());
        assert_eq!(instance.status(), InstanceStatus::Running);
    }
}
