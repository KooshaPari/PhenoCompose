//! Rust FFI bindings to NVMS Go Core
//!
//! These bindings allow Rust code (like PhenoCompose) to call
//! into the NVMS Go library via CGO.
//!
//! # Building
//!
//! First, build the Go library:
//! ```bash
//! cd bindings/go-c-export
//! go build -buildmode=c-archive -o nvms_core.a .
//! ```
//!
//! Then build this crate:
//! ```bash
//! cargo build -p nvms-ffi
//! ```

use std::ffi::{c_char, c_int, c_uint64_t};

mod sys {
    use std::os::raw::{c_char, c_int, c_uint64_t};

    #[repr(C)]
    pub enum NvmsTier {
        WASM = 1,
        GVISOR = 2,
        FIRECRACKER = 3,
    }

    #[repr(C)]
    pub enum NvmsStatus {
        Stopped = 0,
        Starting = 1,
        Running = 2,
        Stopping = 3,
        Error = 4,
    }

    #[repr(C)]
    pub struct NvmsInstance {
        pub id: c_uint64_t,
        pub tier: NvmsTier,
        pub status: NvmsStatus,
        pub name: *const c_char,
    }

    // Go-exported functions
    extern "C" {
        pub fn nvms_version() -> *const c_char;
        pub fn nvms_init() -> c_int;
        pub fn nvms_instance_create(tier: NvmsTier, name: *const c_char) -> *mut NvmsInstance;
        pub fn nvms_instance_destroy(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_start(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_stop(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_status(inst: *mut NvmsInstance) -> NvmsStatus;
    }
}

/// NVMS instance wrapper
pub struct Instance {
    ptr: *mut sys::NvmsInstance,
}

impl Instance {
    /// Create a new NVMS instance
    ///
    /// # Safety
    ///
    /// The returned instance must be destroyed via `destroy()`
    pub unsafe fn create(tier: Tier, name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;
        let ptr = sys::nvms_instance_create(tier.into(), c_name.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Start the instance
    pub fn start(&self) -> Result<(), NvmsError> {
        let ret = unsafe { sys::nvms_instance_start(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(NvmsError::StartFailed)
        }
    }

    /// Stop the instance
    pub fn stop(&self) -> Result<(), NvmsError> {
        let ret = unsafe { sys::nvms_instance_stop(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(NvmsError::StopFailed)
        }
    }

    /// Get instance status
    pub fn status(&self) -> Status {
        unsafe { sys::nvms_instance_status(self.ptr).into() }
    }

    /// Get instance ID
    pub fn id(&self) -> u64 {
        unsafe { (*self.ptr).id as u64 }
    }

    /// Get instance tier
    pub fn tier(&self) -> Tier {
        unsafe { (*self.ptr).tier.into() }
    }

    /// Get instance name
    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = (*self.ptr).name;
            if name_ptr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(name_ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            sys::nvms_instance_destroy(self.ptr);
        }
    }
}

/// Tier levels for isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Tier 1: WASM sandbox (~1ms startup)
    Wasm = 1,
    /// Tier 2: gVisor container (~90ms startup)
    Gvisor = 2,
    /// Tier 3: Firecracker microVM (~125ms startup)
    Firecracker = 3,
}

impl From<Tier> for sys::NvmsTier {
    fn from(tier: Tier) -> Self {
        match tier {
            Tier::Wasm => sys::NvmsTier::WASM,
            Tier::Gvisor => sys::NvmsTier::GVISOR,
            Tier::Firecracker => sys::NvmsTier::FIRECRACKER,
        }
    }
}

impl From<sys::NvmsTier> for Tier {
    fn from(tier: sys::NvmsTier) -> Self {
        match tier {
            sys::NvmsTier::WASM => Tier::Wasm,
            sys::NvmsTier::GVISOR => Tier::Gvisor,
            sys::NvmsTier::FIRECRACKER => Tier::Firecracker,
        }
    }
}

/// Instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl From<sys::NvmsStatus> for Status {
    fn from(status: sys::NvmsStatus) -> Self {
        match status {
            sys::NvmsStatus::Stopped => Status::Stopped,
            sys::NvmsStatus::Starting => Status::Starting,
            sys::NvmsStatus::Running => Status::Running,
            sys::NvmsStatus::Stopping => Status::Stopping,
            sys::NvmsStatus::Error => Status::Error,
        }
    }
}

/// NVMS errors
#[derive(Debug, Clone)]
pub enum NvmsError {
    InitFailed,
    CreateFailed,
    StartFailed,
    StopFailed,
    DestroyFailed,
}

/// Get NVMS version
pub fn version() -> String {
    unsafe {
        let ptr = sys::nvms_version();
        if ptr.is_null() {
            String::from("unknown")
        } else {
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Initialize NVMS
pub fn init() -> Result<(), NvmsError> {
    let ret = unsafe { sys::nvms_init() };
    if ret == 0 {
        Ok(())
    } else {
        Err(NvmsError::InitFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(v.starts_with("1.0"));
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_instance_lifecycle() {
        let inst = unsafe { Instance::create(Tier::Wasm, "test-instance") };
        assert!(inst.is_some());

        let inst = inst.unwrap();
        assert_eq!(inst.tier(), Tier::Wasm);
        assert_eq!(inst.status(), Status::Running);
        assert_eq!(inst.name(), "test-instance");

        assert!(inst.stop().is_ok());
        assert_eq!(inst.status(), Status::Stopped);

        assert!(inst.start().is_ok());
        assert_eq!(inst.status(), Status::Running);
    }

    #[test]
    fn test_tier_conversion() {
        assert_eq!(Tier::Wasm as i32, 1);
        assert_eq!(Tier::Gvisor as i32, 2);
        assert_eq!(Tier::Firecracker as i32, 3);
    }
}
