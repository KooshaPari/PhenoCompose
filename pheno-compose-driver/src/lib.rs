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
//! # fn main() -> Result<(), pheno_compose_driver::DriverError> {
//! let driver = NvmsDriver::new()?;
//! let mut instance = driver.create_instance(Tier::Wasm, "my-service")?;
//! instance.start()?;
//! # Ok(())
//! # }
//! ```

mod config;
mod error;
mod instance;

pub use config::NvmsConfig;
pub use error::DriverError;
pub use instance::{Instance, InstanceStatus, Tier};

pub use nvms_ffi;
use nvms_ffi::Tier as FfiTier;

/// NVMS Driver for PhenoCompose
///
/// Provides high-level access to NVMS 3-tier isolation
pub struct NvmsDriver {
    version: String,
}

impl NvmsDriver {
    /// Create a new NVMS driver
    pub fn new() -> Result<Self, DriverError> {
        nvms_ffi::init().map_err(|source| DriverError::Init { source })?;
        Ok(Self {
            version: nvms_ffi::version(),
        })
    }

    /// Get NVMS version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Create a new instance with the specified tier
    pub fn create_instance(&self, tier: Tier, name: &str) -> Result<Instance, DriverError> {
        let ffi_tier: FfiTier = tier.into();
        let c_name = std::ffi::CString::new(name).map_err(|_| DriverError::CreateInstance {
            tier,
            name: name.to_owned(),
            source: nvms_ffi::NvmsError::CreateFailed,
        })?;
        let ptr = unsafe { nvms_ffi::sys::nvms_instance_create(ffi_tier.into(), c_name.as_ptr()) };
        if ptr.is_null() {
            return Err(DriverError::CreateInstance {
                tier,
                name: name.to_owned(),
                source: nvms_ffi::NvmsError::CreateFailed,
            });
        }
        unsafe { Instance::from_ffi_ptr(ptr) }
    }

    /// Create instance with full configuration
    ///
    /// The configuration is validated before any allocation is
    /// performed. If any field that is not honored by the
    /// underlying NVMS C FFI is set, the call returns
    /// [`DriverError::Config`] listing the offending fields and
    /// no instance is created. The current C ABI accepts only
    /// `tier` and `name`; all other [`NvmsConfig`] fields are
    /// rejected explicitly rather than silently dropped.
    pub fn create_instance_with_config(&self, config: &NvmsConfig) -> Result<Instance, DriverError> {
        config.validate()?;
        self.create_instance(config.tier, &config.name)
    }

    /// List all running instances
    pub fn list_instances(&self) -> Vec<InstanceInfo> {
        nvms_ffi::list_instances()
            .into_iter()
            .map(|(id, tier, status, name)| InstanceInfo {
                id,
                name,
                tier: tier.into(),
                status: status.into(),
            })
            .collect()
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

    #[test]
    fn test_list_instances_empty_initially() {
        let driver = NvmsDriver::new().unwrap();
        let instances = driver.list_instances();
        assert!(
            instances.is_empty(),
            "expected no instances before creation, got {}",
            instances.len()
        );
    }

    #[test]
    fn test_list_instances_after_creation() {
        let driver = NvmsDriver::new().unwrap();

        // Create two instances
        let _wasm = driver.create_instance(Tier::Wasm, "list-test-wasm").unwrap();
        let _fc = driver.create_instance(Tier::Firecracker, "list-test-fc").unwrap();

        let instances = driver.list_instances();
        assert_eq!(instances.len(), 2, "expected 2 instances");

        // Verify instance properties
        let wasm_info = instances.iter().find(|i| i.name == "list-test-wasm");
        assert!(wasm_info.is_some(), "expected 'list-test-wasm' in list");
        assert_eq!(wasm_info.unwrap().tier, Tier::Wasm);

        let fc_info = instances.iter().find(|i| i.name == "list-test-fc");
        assert!(fc_info.is_some(), "expected 'list-test-fc' in list");
        assert_eq!(fc_info.unwrap().tier, Tier::Firecracker);
        assert_eq!(fc_info.unwrap().status, InstanceStatus::Running);
    }

    #[test]
    fn test_list_instances_after_drop() {
        let driver = NvmsDriver::new().unwrap();

        let instance = driver.create_instance(Tier::Gvisor, "drop-test").unwrap();
        assert_eq!(driver.list_instances().len(), 1);

        // Drop instance
        drop(instance);

        let instances = driver.list_instances();
        assert!(
            instances.is_empty(),
            "expected no instances after drop, got {}",
            instances.len()
        );
    }

    // -----------------------------------------------------------------
    // Driver config honesty acceptance tests.
    //
    // The current NVMS C FFI only honors `tier` and `name`. Every
    // other field on `NvmsConfig` must be rejected explicitly by
    // `create_instance_with_config` instead of being silently dropped,
    // so callers get a truthful error before any allocation happens.
    // -----------------------------------------------------------------

    #[test]
    fn config_firecracker_factory_defaults_are_truthfully_rejected() {
        // The `firecracker()` and `firecracker_with()` factories
        // pre-populate cpu_count/memory_bytes from the centralized
        // pheno_config defaults. Those fields are not honored by
        // the current NVMS C FFI, so `create_instance_with_config`
        // must surface a truthful Config error rather than silently
        // ignoring the defaults. Callers wanting the platform
        // defaults should use the lower-level `create_instance`
        // API until the C ABI gains setters for these fields.
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::firecracker("honest-fc-defaults");
        assert!(
            config.cpu_count.is_some() && config.memory_bytes.is_some(),
            "firecracker() factory must still carry the pheno_config defaults \
             so callers can see what would have been requested"
        );
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("firecracker() defaults must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => {
                assert!(
                    msg.contains("cpu_count") && msg.contains("memory_bytes"),
                    "Config error must name both defaulted fields: {msg}"
                );
            }
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert_eq!(
            driver.list_instances().len(),
            baseline,
            "rejected firecracker default config must not allocate"
        );
    }

    #[test]
    fn config_minimal_wasm_is_accepted_and_allocates() {
        let driver = NvmsDriver::new().expect("driver init");
        let config = NvmsConfig::wasm("honest-minimal-wasm");
        assert!(config.validate().is_ok(), "minimal config must validate");
        let instance = driver
            .create_instance_with_config(&config)
            .expect("minimal config allocation succeeds");
        assert_eq!(instance.tier(), Tier::Wasm);
        drop(instance);
    }

    #[test]
    fn config_unsupported_cpu_count_returns_config_error_without_allocation() {
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::wasm("honest-bad-cpus").with_cpus(8);
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("cpu_count must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => {
                assert!(msg.contains("cpu_count"), "Config error must name cpu_count: {msg}");
            }
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert!(
            err.recovery_hint().contains("configuration"),
            "recovery hint must mention configuration"
        );
        assert_eq!(
            driver.list_instances().len(),
            baseline,
            "rejected config must not allocate an instance"
        );
    }

    #[test]
    fn config_unsupported_memory_returns_config_error_without_allocation() {
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::wasm("honest-bad-mem").with_memory_gb(4);
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("memory_bytes must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => {
                assert!(
                    msg.contains("memory_bytes"),
                    "Config error must name memory_bytes: {msg}"
                );
            }
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert_eq!(driver.list_instances().len(), baseline);
    }

    #[test]
    fn config_unsupported_network_returns_config_error_without_allocation() {
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::wasm("honest-bad-net").with_network("default");
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("network must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => assert!(msg.contains("network"), "Config error must name network: {msg}"),
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert_eq!(driver.list_instances().len(), baseline);
    }

    #[test]
    fn config_unsupported_image_returns_config_error_without_allocation() {
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::wasm("honest-bad-image").with_image("ubuntu:22.04");
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("image must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => assert!(msg.contains("image"), "Config error must name image: {msg}"),
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert_eq!(driver.list_instances().len(), baseline);
    }

    #[test]
    fn config_unsupported_env_returns_config_error_without_allocation() {
        let driver = NvmsDriver::new().expect("driver init");
        let baseline = driver.list_instances().len();
        let config = NvmsConfig::wasm("honest-bad-env").with_env("KEY", "VALUE");
        let err = match driver.create_instance_with_config(&config) {
            Ok(_) => panic!("env must be rejected"),
            Err(e) => e,
        };
        match &err {
            DriverError::Config(msg) => assert!(msg.contains("env"), "Config error must name env: {msg}"),
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
        assert_eq!(driver.list_instances().len(), baseline);
    }

    #[test]
    fn config_combined_unsupported_lists_every_field_in_sorted_order() {
        let config = NvmsConfig::wasm("honest-combined")
            .with_env("E", "V")
            .with_image("ubuntu:22.04")
            .with_network("default")
            .with_memory_gb(1)
            .with_cpus(2);
        let unsupported = config.unsupported_fields();
        assert_eq!(
            unsupported,
            vec!["cpu_count", "memory_bytes", "network", "image", "env"],
            "fields must be reported in a stable, sorted order"
        );
        let err = config.validate().expect_err("combined unsupported must fail");
        match err {
            DriverError::Config(msg) => {
                for field in ["cpu_count", "memory_bytes", "network", "image", "env"] {
                    assert!(msg.contains(field), "Config error must name {field}: {msg}");
                }
            }
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
    }

    #[test]
    fn config_empty_unsupported_fields_is_empty_vec() {
        let config = NvmsConfig::wasm("honest-empty");
        assert!(
            config.unsupported_fields().is_empty(),
            "minimal config must report zero unsupported fields"
        );
    }

    #[test]
    fn config_rejected_error_has_no_ffi_source() {
        let config = NvmsConfig::wasm("honest-no-ffi-source").with_cpus(1);
        let err = config.validate().expect_err("must reject cpu_count");
        match err {
            DriverError::Config(_) => {
                // The Config variant must not pretend to be an FFI
                // error so callers don't mistake config issues for
                // backend faults.
            }
            other => panic!("expected DriverError::Config, got {other:?}"),
        }
    }
}
