//! NVMS Configuration

use super::Tier;

/// NVMS Instance Configuration
#[derive(Debug, Clone)]
pub struct NvmsConfig {
    /// Instance name
    pub name: String,

    /// Isolation tier
    pub tier: Tier,

    /// Number of CPUs (optional, for Firecracker)
    pub cpu_count: Option<u32>,

    /// Memory in bytes (optional, for Firecracker)
    pub memory_bytes: Option<u64>,

    /// Network configuration (optional)
    pub network: Option<String>,

    /// Image or binary to run
    pub image: Option<String>,

    /// Environment variables
    pub env: Vec<EnvVar>,
}

/// Environment variable
#[derive(Debug, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl NvmsConfig {
    /// Create a new WASM tier config
    pub fn wasm(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tier: Tier::Wasm,
            cpu_count: None,
            memory_bytes: None,
            network: None,
            image: None,
            env: Vec::new(),
        }
    }

    /// Create a new gVisor tier config
    pub fn gvisor(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tier: Tier::Gvisor,
            cpu_count: None,
            memory_bytes: None,
            network: None,
            image: None,
            env: Vec::new(),
        }
    }

    /// Create a new Firecracker tier config
    pub fn firecracker(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tier: Tier::Firecracker,
            cpu_count: Some(2),
            memory_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
            network: None,
            image: None,
            env: Vec::new(),
        }
    }

    /// Set CPU count
    pub fn with_cpus(mut self, cpus: u32) -> Self {
        self.cpu_count = Some(cpus);
        self
    }

    /// Set memory
    pub fn with_memory_bytes(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Set memory in GB
    pub fn with_memory_gb(self, gb: u64) -> Self {
        self.with_memory_bytes(gb * 1024 * 1024 * 1024)
    }

    /// Set network
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }

    /// Set image
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push(EnvVar {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_config() {
        let config = NvmsConfig::wasm("test-service");
        assert_eq!(config.name, "test-service");
        assert_eq!(config.tier, Tier::Wasm);
    }

    #[test]
    fn test_gvisor_config() {
        let config = NvmsConfig::gvisor("test-service")
            .with_cpus(4)
            .with_memory_gb(4);

        assert_eq!(config.tier, Tier::Gvisor);
        assert_eq!(config.cpu_count, Some(4));
        assert_eq!(config.memory_bytes, Some(4 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_firecracker_config() {
        let config = NvmsConfig::firecracker("prod-service")
            .with_cpus(8)
            .with_memory_gb(16)
            .with_network("default")
            .with_image("ubuntu:22.04")
            .with_env("ENV", "production");

        assert_eq!(config.tier, Tier::Firecracker);
        assert_eq!(config.cpu_count, Some(8));
        assert_eq!(config.memory_bytes, Some(16 * 1024 * 1024 * 1024));
        assert_eq!(config.network, Some("default".into()));
        assert_eq!(config.image, Some("ubuntu:22.04".into()));
        assert_eq!(config.env.len(), 1);
    }
}
