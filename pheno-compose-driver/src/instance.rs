//! NVMS Instance management

use nvms_ffi::{Instance as FfiInstance, NvmsError, Status as FfiStatus, Tier as FfiTier};

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

/// NVMS Instance wrapper
pub struct Instance {
    inner: FfiInstance,
    tier: Tier,
}

impl Instance {
    /// Create from FFI instance (internal use)
    pub(crate) fn from_ffi(inner: FfiInstance) -> Self {
        let tier = inner.tier().into();
        Self { inner, tier }
    }

    /// Start the instance
    pub fn start(&mut self) -> Result<(), NvmsError> {
        self.inner.start()
    }

    /// Stop the instance
    pub fn stop(&mut self) -> Result<(), NvmsError> {
        self.inner.stop()
    }

    /// Get instance status
    pub fn status(&self) -> InstanceStatus {
        self.inner.status().into()
    }

    /// Get instance ID
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    /// Get instance tier
    pub fn tier(&self) -> Tier {
        self.tier
    }

    /// Get instance name
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Check if instance is running
    pub fn is_running(&self) -> bool {
        self.status() == InstanceStatus::Running
    }

    /// Get startup time estimate based on tier
    pub fn estimated_startup_ms(&self) -> u32 {
        match self.tier {
            Tier::Wasm => 1,
            Tier::Gvisor => 90,
            Tier::Firecracker => 125,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // FFI instance is automatically destroyed when dropped
    }
}
