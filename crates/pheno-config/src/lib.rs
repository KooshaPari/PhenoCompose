// SPDX-License-Identifier: MIT OR Apache-2.0
//! Centralized configuration for PhenoCompose.
//!
//! Provides a layered config loader using [`figment`]:
//! 1. Hard-coded Rust defaults
//! 2. `PhenoCompose.toml` config file (optional, CWD)
//! 3. Environment variables prefixed with `PHENO_`
//!
//! # Key groupings
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`nvms`] | NVMS driver version / platform labels |
//! | [`sandbox`] | Sandbox default resources & limits |
//! | [`perf`] | Performance simulation defaults |
//! | [`gpu`] | GPU device defaults |
//!
//! # Example
//!
//! ```rust
//! use pheno_config::PhenoConfig;
//!
//! let cfg = PhenoConfig::load().expect("config loaded");
//! assert!(cfg.sandbox.max_sandbox_id_len >= 1);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use figment::providers::{Env, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use nvms::NvmsConfig;
pub use sandbox::SandboxConfig;
pub use gpu::GpuConfig;
pub use perf::PerfConfig;

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// PhenoCompose top-level configuration.
///
/// Loaded via [`PhenoConfig::load`] which merges:
/// - Hard-coded Rust defaults
/// - `PhenoCompose.toml` (optional) in the current directory
/// - Environment variables prefixed with `PHENO_`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhenoConfig {
    /// NVMS driver / platform labels.
    #[serde(default)]
    pub nvms: NvmsConfig,

    /// Sandbox defaults and resource limits.
    #[serde(default)]
    pub sandbox: SandboxConfig,

    /// Performance simulation defaults.
    #[serde(default)]
    pub perf: PerfConfig,

    /// GPU device defaults.
    #[serde(default)]
    pub gpu: GpuConfig,

    /// Driver-level defaults.
    #[serde(default)]
    pub driver: DriverConfig,
}

/// Driver-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverConfig {
    /// Number of vCPUs for Firecracker instances by default.
    #[serde(default = "default_firecracker_cpus")]
    pub firecracker_default_cpus: u32,

    /// Memory in bytes for Firecracker instances by default.
    #[serde(default = "default_firecracker_memory")]
    pub firecracker_default_memory_bytes: u64,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            firecracker_default_cpus: default_firecracker_cpus(),
            firecracker_default_memory_bytes: default_firecracker_memory(),
        }
    }
}

const fn default_firecracker_cpus() -> u32 { 2 }
const fn default_firecracker_memory() -> u64 { 2 * 1024 * 1024 * 1024 }

// ---------------------------------------------------------------------------
// NvmsConfig
// ---------------------------------------------------------------------------

pub mod nvms {
    //! NVMS driver identification labels.

    use serde::{Deserialize, Serialize};

    /// Labels for the NVMS driver.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NvmsConfig {
        /// Version string reported by `nvms_version()`.
        #[serde(default = "default_version")]
        pub version: String,

        /// Platform info string reported by `nvms_platform_info()`.
        #[serde(default = "default_platform")]
        pub platform: String,
    }

    impl Default for NvmsConfig {
        fn default() -> Self {
            Self {
                version: default_version(),
                platform: default_platform(),
            }
        }
    }

    fn default_version() -> String { "1.0.0".to_string() }
    fn default_platform() -> String { format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH) }
}

// ---------------------------------------------------------------------------
// SandboxConfig
// ---------------------------------------------------------------------------

pub mod sandbox {
    //! Sandbox resource defaults and constraints.

    use serde::{Deserialize, Serialize};

    /// Sandbox-level configuration.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SandboxConfig {
        /// Maximum allowed length for a [`SandboxID`].
        #[serde(default = "default_max_sandbox_id_len")]
        pub max_sandbox_id_len: usize,

        /// Estimated startup time in ms for a Wasm tier instance.
        #[serde(default = "default_wasm_startup_ms")]
        pub startup_ms_wasm: u32,

        /// Estimated startup time in ms for a gVisor tier instance.
        #[serde(default = "default_gvisor_startup_ms")]
        pub startup_ms_gvisor: u32,

        /// Estimated startup time in ms for a Firecracker tier instance.
        #[serde(default = "default_firecracker_startup_ms")]
        pub startup_ms_firecracker: u32,
    }

    impl Default for SandboxConfig {
        fn default() -> Self {
            Self {
                max_sandbox_id_len: default_max_sandbox_id_len(),
                startup_ms_wasm: default_wasm_startup_ms(),
                startup_ms_gvisor: default_gvisor_startup_ms(),
                startup_ms_firecracker: default_firecracker_startup_ms(),
            }
        }
    }

    const fn default_max_sandbox_id_len() -> usize { 128 }
    const fn default_wasm_startup_ms() -> u32 { 1 }
    const fn default_gvisor_startup_ms() -> u32 { 90 }
    const fn default_firecracker_startup_ms() -> u32 { 125 }
}

// ---------------------------------------------------------------------------
// PerfConfig
// ---------------------------------------------------------------------------

pub mod perf {
    //! Performance simulation defaults (used by the in-process shim).

    use serde::{Deserialize, Serialize};

    /// Performance statistics defaults.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PerfConfig {
        /// Simulated startup time in nanoseconds.
        #[serde(default = "default_startup_ns")]
        pub startup_time_ns: u64,

        /// Simulated memory used in bytes.
        #[serde(default = "default_memory_bytes")]
        pub memory_used_bytes: u64,

        /// Simulated GPU utilization (0.0 – 1.0).
        #[serde(default = "default_gpu_utilization")]
        pub gpu_utilization: f64,
    }

    impl Default for PerfConfig {
        fn default() -> Self {
            Self {
                startup_time_ns: default_startup_ns(),
                memory_used_bytes: default_memory_bytes(),
                gpu_utilization: default_gpu_utilization(),
            }
        }
    }

    const fn default_startup_ns() -> u64 { 1_000_000 }
    const fn default_memory_bytes() -> u64 { 64 * 1024 * 1024 }
    const fn default_gpu_utilization() -> f64 { 0.0 }
}

// ---------------------------------------------------------------------------
// GpuConfig
// ---------------------------------------------------------------------------

pub mod gpu {
    //! GPU device defaults.

    use serde::{Deserialize, Serialize};

    /// GPU device configuration defaults.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GpuConfig {
        /// Simulated GPU memory in bytes.
        #[serde(default = "default_gpu_memory_bytes")]
        pub memory_bytes: u64,

        /// Number of compute units / CUDA cores.
        #[serde(default = "default_compute_units")]
        pub compute_units: u32,
    }

    impl Default for GpuConfig {
        fn default() -> Self {
            Self {
                memory_bytes: default_gpu_memory_bytes(),
                compute_units: default_compute_units(),
            }
        }
    }

    const fn default_gpu_memory_bytes() -> u64 { 8 * 1024 * 1024 * 1024 }
    const fn default_compute_units() -> u32 { 8 }
}

// ---------------------------------------------------------------------------
// Combined defaults
// ---------------------------------------------------------------------------

impl Default for PhenoConfig {
    fn default() -> Self {
        Self {
            nvms: NvmsConfig::default(),
            sandbox: SandboxConfig::default(),
            perf: PerfConfig::default(),
            gpu: GpuConfig::default(),
            driver: DriverConfig::default(),
        }
    }
}

impl PhenoConfig {
    /// Load configuration using figment's layered providers:
    ///
    /// 1. Hard-coded Rust defaults (via [`Serialized`])
    /// 2. Optional `PhenoCompose.toml` in the current directory
    /// 3. Environment variables prefixed with `PHENO_`
    ///
    /// Later providers override earlier ones.
    ///
    /// # Errors
    ///
    /// Returns [`figment::Error`] if the TOML file exists but is
    /// malformed, or if env-var parsing fails.
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Serialized::defaults(PhenoConfig::default()))
            .merge(Toml::file("PhenoCompose.toml").nested())
            .merge(Env::prefixed("PHENO_").global().nested())
            .extract()
    }

    /// Load configuration, panicking on load errors.
    ///
    /// Convenience for `init` / `main` contexts where a missing
    /// config file is a hard failure.
    pub fn load_or_panic() -> Self {
        Self::load().expect("PhenoConfig: failed to load (check PhenoCompose.toml or PHENO_* env vars)")
    }

    /// Return only the parsed defaults (ignores file and env
    /// sources).  Useful in tests.
    pub fn defaults_only() -> Self {
        PhenoConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Builder-style override
// ---------------------------------------------------------------------------

impl PhenoConfig {
    /// Replace the [`NvmsConfig`] section.
    #[must_use]
    pub fn with_nvms(mut self, nvms: NvmsConfig) -> Self {
        self.nvms = nvms;
        self
    }

    /// Replace the [`SandboxConfig`] section.
    #[must_use]
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Replace the [`PerfConfig`] section.
    #[must_use]
    pub fn with_perf(mut self, perf: PerfConfig) -> Self {
        self.perf = perf;
        self
    }

    /// Replace the [`GpuConfig`] section.
    #[must_use]
    pub fn with_gpu(mut self, gpu: GpuConfig) -> Self {
        self.gpu = gpu;
        self
    }

    /// Replace the [`DriverConfig`] section.
    #[must_use]
    pub fn with_driver(mut self, driver: DriverConfig) -> Self {
        self.driver = driver;
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Defaults -----------------------------------------------------------

    #[test]
    fn default_config_has_sane_nvms_values() {
        let cfg = PhenoConfig::default();
        assert!(!cfg.nvms.version.is_empty(), "version must not be empty");
        assert!(!cfg.nvms.platform.is_empty(), "platform must not be empty");
        assert!(cfg.nvms.platform.contains('/'), "platform should contain '/'");
    }

    #[test]
    fn default_config_has_sane_sandbox_values() {
        let cfg = PhenoConfig::default();
        assert_eq!(cfg.sandbox.max_sandbox_id_len, 128);
        assert_eq!(cfg.sandbox.startup_ms_wasm, 1);
        assert_eq!(cfg.sandbox.startup_ms_gvisor, 90);
        assert_eq!(cfg.sandbox.startup_ms_firecracker, 125);
    }

    #[test]
    fn default_config_has_sane_driver_values() {
        let cfg = PhenoConfig::default();
        assert_eq!(cfg.driver.firecracker_default_cpus, 2);
        assert_eq!(cfg.driver.firecracker_default_memory_bytes, 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn default_config_has_sane_perf_values() {
        let cfg = PhenoConfig::default();
        assert_eq!(cfg.perf.startup_time_ns, 1_000_000);
        assert_eq!(cfg.perf.memory_used_bytes, 64 * 1024 * 1024);
        assert!((cfg.perf.gpu_utilization - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_config_has_sane_gpu_values() {
        let cfg = PhenoConfig::default();
        assert_eq!(cfg.gpu.memory_bytes, 8 * 1024 * 1024 * 1024);
        assert_eq!(cfg.gpu.compute_units, 8);
    }

    // -- Load without file --------------------------------------------------

    #[test]
    fn load_works_without_config_file() {
        // When no PhenoCompose.toml is present, defaults should be used.
        let cfg = PhenoConfig::load().expect("load should succeed without file");
        assert_eq!(cfg.sandbox.max_sandbox_id_len, 128);
    }

    // -- Builder overrides --------------------------------------------------

    #[test]
    fn builder_overrides_sandbox() {
        let sb = sandbox::SandboxConfig {
            max_sandbox_id_len: 64,
            ..Default::default()
        };
        let cfg = PhenoConfig::default().with_sandbox(sb);
        assert_eq!(cfg.sandbox.max_sandbox_id_len, 64);
        assert_eq!(cfg.sandbox.startup_ms_wasm, 1); // unchanged
    }

    #[test]
    fn builder_overrides_driver() {
        let drv = DriverConfig {
            firecracker_default_cpus: 4,
            firecracker_default_memory_bytes: 4 * 1024 * 1024 * 1024,
        };
        let cfg = PhenoConfig::default().with_driver(drv);
        assert_eq!(cfg.driver.firecracker_default_cpus, 4);
        assert_eq!(cfg.driver.firecracker_default_memory_bytes, 4 * 1024 * 1024 * 1024);
    }

    // -- Serialization round-trip ------------------------------------------

    #[test]
    fn default_config_round_trips_via_serde() {
        let cfg = PhenoConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: PhenoConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.sandbox.max_sandbox_id_len, cfg.sandbox.max_sandbox_id_len);
        assert_eq!(
            deserialized.driver.firecracker_default_cpus,
            cfg.driver.firecracker_default_cpus,
        );
    }
}
