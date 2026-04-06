//! Rust FFI bindings to NVMS Go Core
//!
//! Complete FFI bindings with GPU support for:
//! - Apple Silicon (Metal, Unified Memory, ANE)
//! - NVIDIA (CUDA, Tensor Cores, Unified Memory)
//! - AMD (ROCm, Matrix Cores)
//! - ARM64 NEON/SIMD optimizations

use std::ffi::{c_char, c_int, c_uint64_t};

// C types from Go
mod sys {
    use std::os::raw::{c_char, c_int, c_uint64_t};

    #[repr(C)]
    pub enum NvmsTier {
        Wasm = 1,
        Gvisor = 2,
        Firecracker = 3,
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
    pub enum NvmsGpuBackend {
        None = 0,
        AppleMetal = 1,
        NvidiaCuda = 2,
        AmdRocm = 3,
        IntelOneApi = 4,
    }

    #[repr(C)]
    pub enum NvmsMemoryType {
        Cpu = 0,
        Gpu = 1,
        Unified = 2,
    }

    #[repr(C)]
    pub struct NvmsInstance {
        pub id: c_uint64_t,
        pub tier: NvmsTier,
        pub status: NvmsStatus,
        pub name: *const c_char,
        pub gpu_backend: NvmsGpuBackend,
        pub memory_type: NvmsMemoryType,
        pub gpu_memory_bytes: c_uint64_t,
    }

    #[repr(C)]
    pub struct NvmsGpuDevice {
        pub name: [c_char; 256],
        pub backend: NvmsGpuBackend,
        pub memory_bytes: c_uint64_t,
        pub compute_units: c_int,
        pub supports_unified_memory: bool,
    }

    #[repr(C)]
    pub struct NvmsPerfStats {
        pub startup_time_ns: c_uint64_t,
        pub memory_used_bytes: c_uint64_t,
        pub gpu_utilization: f64,
    }

    extern "C" {
        pub fn nvms_version() -> *const c_char;
        pub fn nvms_platform_info() -> *const c_char;
        pub fn nvms_init() -> c_int;
        pub fn nvms_init_gpu(backend: NvmsGpuBackend) -> c_int;

        pub fn nvms_gpu_info() -> NvmsGpuDevice;
        pub fn nvms_supports_gpu() -> bool;
        pub fn nvms_supports_unified_memory() -> bool;

        pub fn nvms_apple_silicon_init() -> c_int;
        pub fn nvms_apple_ane_available() -> bool;
        pub fn nvms_apple_unified_memory_alloc(size: c_uint64_t) -> *mut std::ffi::c_void;

        pub fn nvms_cuda_init() -> c_int;
        pub fn nvms_cuda_device_count() -> c_int;
        pub fn nvms_cuda_alloc_unified(size: c_uint64_t) -> *mut std::ffi::c_void;

        pub fn nvms_rocm_init() -> c_int;
        pub fn nvms_rocm_device_count() -> c_int;

        pub fn nvms_neon_available() -> bool;

        pub fn nvms_instance_create(tier: NvmsTier, name: *const c_char) -> *mut NvmsInstance;
        pub fn nvms_instance_destroy(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_start(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_stop(inst: *mut NvmsInstance) -> c_int;
        pub fn nvms_instance_status(inst: *mut NvmsInstance) -> NvmsStatus;
        pub fn nvms_perf_stats() -> NvmsPerfStats;
    }
}

/// NVMS version
pub fn version() -> String {
    unsafe {
        let ptr = sys::nvms_version();
        cstr_to_string(ptr)
    }
}

/// Platform info (e.g., "darwin/arm64", "linux/amd64")
pub fn platform_info() -> String {
    unsafe {
        let ptr = sys::nvms_platform_info();
        cstr_to_string(ptr)
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

/// GPU backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    None,
    AppleMetal,
    NvidiaCuda,
    AmdRocm,
    IntelOneApi,
}

impl From<GpuBackend> for sys::NvmsGpuBackend {
    fn from(backend: GpuBackend) -> Self {
        match backend {
            GpuBackend::None => sys::NvmsGpuBackend::None,
            GpuBackend::AppleMetal => sys::NvmsGpuBackend::AppleMetal,
            GpuBackend::NvidiaCuda => sys::NvmsGpuBackend::NvidiaCuda,
            GpuBackend::AmdRocm => sys::NvmsGpuBackend::AmdRocm,
            GpuBackend::IntelOneApi => sys::NvmsGpuBackend::IntelOneApi,
        }
    }
}

impl From<sys::NvmsGpuBackend> for GpuBackend {
    fn from(backend: sys::NvmsGpuBackend) -> Self {
        match backend {
            sys::NvmsGpuBackend::None => GpuBackend::None,
            sys::NvmsGpuBackend::AppleMetal => GpuBackend::AppleMetal,
            sys::NvmsGpuBackend::NvidiaCuda => GpuBackend::NvidiaCuda,
            sys::NvmsGpuBackend::AmdRocm => GpuBackend::AmdRocm,
            sys::NvmsGpuBackend::IntelOneApi => GpuBackend::IntelOneApi,
        }
    }
}

/// GPU device info
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub backend: GpuBackend,
    pub memory_bytes: u64,
    pub compute_units: u32,
    pub supports_unified_memory: bool,
}

/// Get GPU info
pub fn gpu_info() -> GpuDevice {
    unsafe {
        let dev = sys::nvms_gpu_info();
        GpuDevice {
            name: cstr_to_string(&dev.name as *const c_char),
            backend: dev.backend.into(),
            memory_bytes: dev.memory_bytes as u64,
            compute_units: dev.compute_units as u32,
            supports_unified_memory: dev.supports_unified_memory,
        }
    }
}

/// Check if GPU is available
pub fn supports_gpu() -> bool {
    unsafe { sys::nvms_supports_gpu() }
}

/// Check if unified memory is supported
pub fn supports_unified_memory() -> bool {
    unsafe { sys::nvms_supports_unified_memory() }
}

/// Apple Silicon specific

/// Initialize Apple Silicon optimizations
pub fn apple_silicon_init() -> Result<(), NvmsError> {
    let ret = unsafe { sys::nvms_apple_silicon_init() };
    if ret == 0 {
        Ok(())
    } else {
        Err(NvmsError::AppleSiliconNotSupported)
    }
}

/// Check if Apple Neural Engine is available
pub fn apple_ane_available() -> bool {
    unsafe { sys::nvms_apple_ane_available() }
}

/// Allocate unified memory (Apple Silicon)
pub fn apple_unified_memory_alloc(size: u64) -> *mut std::ffi::c_void {
    unsafe { sys::nvms_apple_unified_memory_alloc(size) }
}

/// CUDA specific

/// Initialize CUDA
pub fn cuda_init() -> Result<(), NvmsError> {
    let ret = unsafe { sys::nvms_cuda_init() };
    if ret == 0 {
        Ok(())
    } else {
        Err(NvmsError::CudaInitFailed)
    }
}

/// Get CUDA device count
pub fn cuda_device_count() -> i32 {
    unsafe { sys::nvms_cuda_device_count() }
}

/// Allocate unified memory (CUDA)
pub fn cuda_alloc_unified(size: u64) -> *mut std::ffi::c_void {
    unsafe { sys::nvms_cuda_alloc_unified(size) }
}

/// ROCm specific

/// Initialize ROCm
pub fn rocm_init() -> Result<(), NvmsError> {
    let ret = unsafe { sys::nvms_rocm_init() };
    if ret == 0 {
        Ok(())
    } else {
        Err(NvmsError::RocmInitFailed)
    }
}

/// Get ROCm device count
pub fn rocm_device_count() -> i32 {
    unsafe { sys::nvms_rocm_device_count() }
}

/// ARM64 NEON available
pub fn neon_available() -> bool {
    unsafe { sys::nvms_neon_available() }
}

/// Tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Wasm = 1,
    Gvisor = 2,
    Firecracker = 3,
}

impl From<Tier> for sys::NvmsTier {
    fn from(tier: Tier) -> Self {
        match tier {
            Tier::Wasm => sys::NvmsTier::Wasm,
            Tier::Gvisor => sys::NvmsTier::Gvisor,
            Tier::Firecracker => sys::NvmsTier::Firecracker,
        }
    }
}

impl From<sys::NvmsTier> for Tier {
    fn from(tier: sys::NvmsTier) -> Self {
        match tier {
            sys::NvmsTier::Wasm => Tier::Wasm,
            sys::NvmsTier::Gvisor => Tier::Gvisor,
            sys::NvmsTier::Firecracker => Tier::Firecracker,
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

/// Memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Cpu,
    Gpu,
    Unified,
}

impl From<sys::NvmsMemoryType> for MemoryType {
    fn from(mem: sys::NvmsMemoryType) -> Self {
        match mem {
            sys::NvmsMemoryType::Cpu => MemoryType::Cpu,
            sys::NvmsMemoryType::Gpu => MemoryType::Gpu,
            sys::NvmsMemoryType::Unified => MemoryType::Unified,
        }
    }
}

/// NVMS instance
pub struct Instance {
    ptr: *mut sys::NvmsInstance,
}

impl Instance {
    /// Create a new instance
    pub unsafe fn create(tier: Tier, name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;
        let ptr = sys::nvms_instance_create(tier.into(), c_name.as_ptr());
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn start(&self) -> Result<(), NvmsError> {
        let ret = unsafe { sys::nvms_instance_start(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(NvmsError::StartFailed)
        }
    }

    pub fn stop(&self) -> Result<(), NvmsError> {
        let ret = unsafe { sys::nvms_instance_stop(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(NvmsError::StopFailed)
        }
    }

    pub fn status(&self) -> Status {
        unsafe { sys::nvms_instance_status(self.ptr).into() }
    }

    pub fn tier(&self) -> Tier {
        unsafe { (*self.ptr).tier.into() }
    }

    pub fn gpu_backend(&self) -> GpuBackend {
        unsafe { (*self.ptr).gpu_backend.into() }
    }

    pub fn memory_type(&self) -> MemoryType {
        unsafe { (*self.ptr).memory_type.into() }
    }

    pub fn id(&self) -> u64 {
        unsafe { (*self.ptr).id as u64 }
    }

    pub fn name(&self) -> String {
        unsafe {
            let ptr = (*self.ptr).name;
            if ptr.is_null() {
                String::new()
            } else {
                cstr_to_string(ptr)
            }
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { sys::nvms_instance_destroy(self.ptr) };
    }
}

/// Performance stats
#[derive(Debug, Clone)]
pub struct PerfStats {
    pub startup_time_ns: u64,
    pub memory_used_bytes: u64,
    pub gpu_utilization: f64,
}

pub fn perf_stats() -> PerfStats {
    unsafe {
        let stats = sys::nvms_perf_stats();
        PerfStats {
            startup_time_ns: stats.startup_time_ns as u64,
            memory_used_bytes: stats.memory_used_bytes as u64,
            gpu_utilization: stats.gpu_utilization,
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
    AppleSiliconNotSupported,
    CudaInitFailed,
    RocmInitFailed,
}

fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
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
    fn test_platform() {
        let p = platform_info();
        assert!(p.contains('/'));
    }

    #[test]
    fn test_gpu_support() {
        let has_gpu = supports_gpu();
        println!("GPU available: {}", has_gpu);

        if has_gpu {
            let info = gpu_info();
            println!("GPU backend: {:?}", info.backend);
        }
    }

    #[test]
    fn test_apple_silicon() {
        let has_ane = apple_ane_available();
        println!("Apple ANE available: {}", has_ane);

        let has_neon = neon_available();
        println!("NEON available: {}", has_neon);
    }

    #[test]
    fn test_instance() {
        init().unwrap();

        let inst = unsafe { Instance::create(Tier::Wasm, "test") };
        assert!(inst.is_some());

        let inst = inst.unwrap();
        assert_eq!(inst.tier(), Tier::Wasm);
        assert_eq!(inst.status(), Status::Running);

        inst.stop().unwrap();
        assert_eq!(inst.status(), Status::Stopped);

        inst.start().unwrap();
        assert_eq!(inst.status(), Status::Running);
    }
}
