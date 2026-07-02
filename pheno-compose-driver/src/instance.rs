// SPDX-License-Identifier: MIT OR Apache-2.0
//! NVMS Instance management

use std::ptr::NonNull;

use nvms_ffi::{Status as FfiStatus, Tier as FfiTier};

use crate::DriverError;

/// Instance tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Tier 1: WASM sandbox (~1ms startup)
    Wasm,
    /// Tier 2: gVisor container (~90ms startup)
    Gvisor,
    /// Tier 3: Firecracker microVM (~125ms startup)
    Firecracker,
}

impl From<nvms_ffi::sys::NvmsTier> for Tier {
    fn from(tier: nvms_ffi::sys::NvmsTier) -> Self {
        match tier {
            nvms_ffi::sys::NvmsTier::Wasm => Tier::Wasm,
            nvms_ffi::sys::NvmsTier::Gvisor => Tier::Gvisor,
            nvms_ffi::sys::NvmsTier::Firecracker => Tier::Firecracker,
        }
    }
}

impl From<Tier> for FfiTier {
    fn from(tier: Tier) -> Self {
        match tier {
            Tier::Wasm => FfiTier::Wasm,
            Tier::Gvisor => FfiTier::Gvisor,
            Tier::Firecracker => FfiTier::Firecracker,
        }
    }
}

impl From<FfiTier> for Tier {
    fn from(tier: FfiTier) -> Self {
        match tier {
            FfiTier::Wasm => Tier::Wasm,
            FfiTier::Gvisor => Tier::Gvisor,
            FfiTier::Firecracker => Tier::Firecracker,
        }
    }
}

/// Instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl From<nvms_ffi::sys::NvmsStatus> for InstanceStatus {
    fn from(status: nvms_ffi::sys::NvmsStatus) -> Self {
        match status {
            nvms_ffi::sys::NvmsStatus::Stopped => InstanceStatus::Stopped,
            nvms_ffi::sys::NvmsStatus::Starting => InstanceStatus::Starting,
            nvms_ffi::sys::NvmsStatus::Running => InstanceStatus::Running,
            nvms_ffi::sys::NvmsStatus::Stopping => InstanceStatus::Stopping,
            nvms_ffi::sys::NvmsStatus::Error => InstanceStatus::Error,
        }
    }
}

impl From<FfiStatus> for InstanceStatus {
    fn from(status: FfiStatus) -> Self {
        match status {
            FfiStatus::Stopped => InstanceStatus::Stopped,
            FfiStatus::Starting => InstanceStatus::Starting,
            FfiStatus::Running => InstanceStatus::Running,
            FfiStatus::Stopping => InstanceStatus::Stopping,
            FfiStatus::Error => InstanceStatus::Error,
        }
    }
}

/// NVMS Instance wrapper with safe FFI boundary
pub struct Instance {
    inner: NonNull<nvms_ffi::sys::NvmsInstance>,
    tier: Tier,
}

impl Instance {
    /// Create from FFI instance (internal use)
    ///
    /// # Safety
    /// The pointer must be non-null and valid for the lifetime of the Instance.
    pub(crate) unsafe fn from_ffi_ptr(ptr: *mut nvms_ffi::sys::NvmsInstance) -> Result<Self, DriverError> {
        let inner = NonNull::new(ptr).ok_or_else(|| DriverError::CreateInstance {
            tier: crate::Tier::Wasm,
            name: String::new(),
            source: nvms_ffi::NvmsError::CreateFailed,
        })?;
        let tier = (*ptr).tier.into();
        Ok(Self { inner, tier })
    }

    /// Start the instance
    pub fn start(&mut self) -> Result<(), DriverError> {
        let id = self.id();
        let code = unsafe { nvms_ffi::sys::nvms_instance_start(self.inner.as_ptr()) };
        if code != 0 {
            return Err(DriverError::Start {
                instance_id: id,
                source: nvms_ffi::NvmsError::from(code),
            });
        }
        Ok(())
    }

    /// Stop the instance
    pub fn stop(&mut self) -> Result<(), DriverError> {
        let id = self.id();
        let code = unsafe { nvms_ffi::sys::nvms_instance_stop(self.inner.as_ptr()) };
        if code != 0 {
            return Err(DriverError::Stop {
                instance_id: id,
                source: nvms_ffi::NvmsError::from(code),
            });
        }
        Ok(())
    }

    /// Get instance status
    pub fn status(&self) -> InstanceStatus {
        unsafe { nvms_ffi::sys::nvms_instance_status(self.inner.as_ptr()).into() }
    }

    /// Get instance ID
    pub fn id(&self) -> u64 {
        unsafe { (*self.inner.as_ptr()).id }
    }

    /// Get instance tier
    pub fn tier(&self) -> Tier {
        self.tier
    }

    /// Get instance name
    pub fn name(&self) -> String {
        unsafe {
            let ptr = (*self.inner.as_ptr()).name;
            if ptr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    /// Check if instance is running
    pub fn is_running(&self) -> bool {
        self.status() == InstanceStatus::Running
    }

    /// Get startup time estimate based on tier
    ///
    /// Caches the config in a thread-local OnceCell to avoid paying
    /// the Figment load cost on every call.
    pub fn estimated_startup_ms(&self) -> u32 {
        use std::cell::OnceCell;
        thread_local! {
            static CFG: OnceCell<pheno_config::PhenoConfig> = const { OnceCell::new() };
        }
        CFG.with(|c| {
            let cfg = c.get_or_init(pheno_config::PhenoConfig::default);
            match self.tier {
                Tier::Wasm => cfg.sandbox.startup_ms_wasm,
                Tier::Gvisor => cfg.sandbox.startup_ms_gvisor,
                Tier::Firecracker => cfg.sandbox.startup_ms_firecracker,
            }
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Best-effort cleanup. FFI destroy errors are logged via a
        // raw eprintln because the tracing subscriber may already be
        // shut down by the time Drop runs (e.g. on panic unwinding).
        let code = unsafe { nvms_ffi::sys::nvms_instance_destroy(self.inner.as_ptr()) };
        if code != 0 {
            eprintln!(
                "pheno-compose-driver: nvms_instance_destroy returned {} for instance {}",
                code,
                self.id()
            );
        }
    }
}
