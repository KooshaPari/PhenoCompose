// SPDX-License-Identifier: MIT OR Apache-2.0
//! NVMS Instance management

use std::ptr::NonNull;

use nvms_ffi::{NvmsError, Status as FfiStatus, Tier as FfiTier};

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
    pub(crate) unsafe fn from_ffi_ptr(ptr: *mut nvms_ffi::sys::NvmsInstance) -> Result<Self, NvmsError> {
        let inner = NonNull::new(ptr).ok_or(NvmsError::CreateFailed)?;
        let tier = (*ptr).tier.into();
        Ok(Self { inner, tier })
    }

    /// Start the instance
    pub fn start(&mut self) -> Result<(), NvmsError> {
        unsafe { nvms_ffi::sys::nvms_instance_start(self.inner.as_ptr()) };
        Ok(())
    }

    /// Stop the instance
    pub fn stop(&mut self) -> Result<(), NvmsError> {
        unsafe { nvms_ffi::sys::nvms_instance_stop(self.inner.as_ptr()) };
        Ok(())
    }

    /// Get instance status
    pub fn status(&self) -> InstanceStatus {
        unsafe { nvms_ffi::sys::nvms_instance_status(self.inner.as_ptr()).into() }
    }

    /// Get instance ID
    pub fn id(&self) -> u64 {
        unsafe { (*self.inner.as_ptr()).id as u64 }
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
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Check if instance is running
    pub fn is_running(&self) -> bool {
        self.status() == InstanceStatus::Running
    }

    /// Get startup time estimate based on tier
    pub fn estimated_startup_ms(&self) -> u32 {
        let cfg = pheno_config::PhenoConfig::default();
        match self.tier {
            Tier::Wasm => cfg.sandbox.startup_ms_wasm,
            Tier::Gvisor => cfg.sandbox.startup_ms_gvisor,
            Tier::Firecracker => cfg.sandbox.startup_ms_firecracker,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { nvms_ffi::sys::nvms_instance_destroy(self.inner.as_ptr()) };
    }
}
