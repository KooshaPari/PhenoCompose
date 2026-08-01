//! A small, deterministic composition model shared by renderers.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Supported output targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// Docker Compose.
    Docker,
    /// Kubernetes manifests.
    Kubernetes,
    /// process-compose.
    Process,
    /// NanoVMS execution plan.
    NanoVms,
}

/// Execution substrates that may consume a rendered plan.
///
/// These are runtime adapters, not cloud providers: they do not own
/// deployment state, credentials, or infrastructure policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExecutionBackend {
    /// NanoVMS tiered execution engine.
    NanoVms,
    /// Podman-compatible OCI container runtime.
    Podman,
    /// Apple Containers extension for OCI workloads.
    AppleContainers,
    /// First-party WSL containers extension.
    WslContainers,
}

impl ExecutionBackend {
    /// Returns whether this backend can consume the target's plan format.
    pub fn supports(self, target: Target) -> bool {
        match self {
            Self::NanoVms => target == Target::NanoVms,
            Self::Podman | Self::AppleContainers | Self::WslContainers => target == Target::Docker,
        }
    }
}

impl Target {
    fn content_type(self) -> &'static str {
        match self {
            Self::Docker => "application/vnd.docker.compose+yaml",
            Self::Kubernetes => "application/vnd.kubernetes+yaml",
            Self::Process => "application/vnd.process-compose+yaml",
            Self::NanoVms => "application/vnd.nanovms.plan+json",
        }
    }
}

/// A service in a composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Optional OCI image.
    pub image: Option<String>,
    /// Optional executable and arguments.
    pub command: Option<Vec<String>>,
    /// Environment values; secret values must be references.
    pub environment: BTreeMap<String, String>,
    /// Published ports.
    pub ports: Vec<Port>,
    /// Optional health check.
    pub health: Option<HealthCheck>,
    /// Resource hints.
    pub resources: Resources,
    /// Startup dependencies.
    pub depends_on: BTreeSet<String>,
}

/// A service port.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Port {
    /// Stable port name.
    pub name: String,
    /// Container port.
    pub container_port: u16,
    /// Transport protocol.
    pub protocol: Protocol,
}

/// Port transport protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
}

/// Health check declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheck {
    /// Check kind.
    pub kind: HealthKind,
    /// Optional endpoint path for HTTP checks.
    pub path: Option<String>,
    /// Optional port.
    pub port: Option<u16>,
}

/// Health check kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthKind {
    /// HTTP GET.
    Http,
    /// TCP connect.
    Tcp,
    /// Process command.
    Command,
}

/// Resource hints passed to a target renderer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resources {
    /// CPU request.
    pub cpu: Option<String>,
    /// Memory request.
    pub memory: Option<String>,
}

/// A complete composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    /// DNS-compatible composition name.
    pub name: String,
    /// Services keyed by stable name.
    pub services: BTreeMap<String, Service>,
    /// Requested render targets.
    pub targets: BTreeSet<Target>,
}

/// A rendered target artifact with a content digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPlan {
    /// Target used.
    pub target: Target,
    /// Composition name.
    pub composition_name: String,
    /// SHA-256 digest of content.
    pub digest: String,
    /// MIME-like content type.
    pub content_type: &'static str,
    /// Rendered bytes as UTF-8.
    pub content: String,
}

/// Cloud-control-plane handoff containing only validated artifact metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BytePortHandoff {
    /// Composition name.
    pub composition_name: String,
    /// Renderer target.
    pub target: Target,
    /// Digest used for apply-time verification.
    pub digest: String,
    /// Rendered artifact bytes.
    pub content: String,
}

/// Execution-engine handoff containing no cloud credentials or state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NanoVmsHandoff {
    /// Composition name.
    pub composition_name: String,
    /// Immutable plan digest.
    pub digest: String,
    /// NanoVMS plan bytes.
    pub content: String,
}

/// Runtime handoff for an explicitly selected execution substrate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionHandoff {
    /// Selected runtime backend.
    pub backend: ExecutionBackend,
    /// Composition name.
    pub composition_name: String,
    /// Immutable plan digest.
    pub digest: String,
    /// Rendered plan bytes.
    pub content: String,
}

/// Desired-state intent sent to the BytePort compute-mesh control plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshWorkloadIntent {
    /// Authenticated owner identity.
    pub owner: String,
    /// Composition name.
    pub composition_name: String,
    /// Immutable render digest.
    pub digest: String,
    /// Artifact or plan locator.
    pub artifact_ref: String,
    /// Runtime substrate hint.
    pub backend: ExecutionBackend,
}

/// JSON envelope for the two provider-facing bridge payloads.
///
/// BytePort and NanoVMS expose different HTTP contracts: the former accepts
/// one desired-state request while the latter accepts one `SandboxConfig` per
/// sandbox. Keeping both exact payload shapes in one envelope lets an offline
/// fixture prove deterministic serialization without pretending that either
/// service accepts the other's fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BridgeContract {
    /// Exact body for `POST /api/v1/mesh/workloads`.
    pub byteport: BytePortMeshWorkloadRequest,
    /// Exact bodies for one or more `POST /v1/deploy` calls.
    pub nanovms: Vec<NanoVmsDeployRequest>,
}

impl BridgeContract {
    /// Construct a bridge envelope from the provider-specific payloads.
    #[must_use]
    pub fn new(byteport: BytePortMeshWorkloadRequest, nanovms: Vec<NanoVmsDeployRequest>) -> Self {
        Self { byteport, nanovms }
    }
}

/// BytePort's authenticated `/api/v1/mesh/workloads` request body.
///
/// The authenticated owner is deliberately absent: BytePort derives it from
/// the request credentials rather than trusting a body field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BytePortMeshWorkloadRequest {
    /// DNS-label-compatible composition name.
    pub name: String,
    /// Immutable rendered-plan digest (`sha256:<64 hex characters>`).
    pub composition_digest: String,
    /// OCI artifact or plan locator.
    pub artifact_ref: String,
    /// BytePort execution-backend identifier.
    pub execution_backend: BytePortExecutionBackend,
    /// Portable placement preferences and constraints.
    pub placement: BytePortPlacement,
}

impl BytePortMeshWorkloadRequest {
    /// Convert an already validated mesh intent to BytePort's wire shape.
    ///
    /// `owner` is intentionally not copied because the BytePort endpoint
    /// obtains it from authentication context.
    #[must_use]
    pub fn from_intent(intent: &MeshWorkloadIntent, placement: BytePortPlacement) -> Self {
        Self {
            name: intent.composition_name.clone(),
            composition_digest: intent.digest.clone(),
            artifact_ref: intent.artifact_ref.clone(),
            execution_backend: intent.backend.into(),
            placement,
        }
    }
}

/// Execution backend names accepted by BytePort's mesh workload endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BytePortExecutionBackend {
    /// NanoVMS execution backend (`nanovms`).
    #[cfg_attr(feature = "serde", serde(rename = "nanovms"))]
    NanoVms,
    /// Podman-compatible OCI backend (`podman`).
    #[cfg_attr(feature = "serde", serde(rename = "podman"))]
    Podman,
    /// Apple Containers backend (`apple-containers`).
    #[cfg_attr(feature = "serde", serde(rename = "apple-containers"))]
    AppleContainers,
    /// WSL containers backend (`wsl-containers`).
    #[cfg_attr(feature = "serde", serde(rename = "wsl-containers"))]
    WslContainers,
}

impl From<ExecutionBackend> for BytePortExecutionBackend {
    fn from(backend: ExecutionBackend) -> Self {
        match backend {
            ExecutionBackend::NanoVms => Self::NanoVms,
            ExecutionBackend::Podman => Self::Podman,
            ExecutionBackend::AppleContainers => Self::AppleContainers,
            ExecutionBackend::WslContainers => Self::WslContainers,
        }
    }
}

/// Portable placement fields accepted by BytePort.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BytePortPlacement {
    /// Optional region preference.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub region: Option<String>,
    /// Optional availability-zone preference.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub zone: Option<String>,
    /// Optional node-pool preference.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub node_pool: Option<String>,
    /// Stable placement labels.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "map_is_empty"))]
    pub labels: BTreeMap<String, String>,
    /// Provider placement constraints.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "map_is_empty"))]
    pub constraints: BTreeMap<String, String>,
}

/// NanoVMS `POST /v1/deploy` request body.
///
/// These fields intentionally mirror the tagged fields of NanoVMS'
/// `domain.SandboxConfig`. The composition digest is not a NanoVMS HTTP field;
/// callers correlate it through the `env` map (for example,
/// `phenocompose.sha256`) when using this endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoVmsDeployRequest {
    /// Stable sandbox name.
    pub name: String,
    /// OCI image reference.
    pub image: String,
    /// NanoVMS VM flavor (`native`, `lima`, `wsl`, `microvm`, or `wasm`).
    pub vm_type: String,
    /// Optional NanoVMS VM tier/flavor override.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub vm_tier: Option<String>,
    /// Optional VM-specific configuration.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub vm_config: Option<NanoVmConfig>,
    /// Isolation type (`vm`, `container`, `wasm`, `process`, or `native`).
    pub sandbox_type: String,
    /// Optional primary sandbox layer.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub sandbox_layer: Option<String>,
    /// Optional stacked isolation layers.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "vec_is_empty"))]
    pub sandbox_layers: Vec<String>,
    /// Optional native-sandbox configuration.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub native_sandbox: Option<NanoNativeSandboxConfig>,
    /// Correlation labels, including composition identity when needed.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "map_is_empty"))]
    pub labels: BTreeMap<String, String>,
    /// Filesystem mounts.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "vec_is_empty"))]
    pub mounts: Vec<NanoMount>,
    /// Environment values (`env` on the NanoVMS wire contract).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "env", default, skip_serializing_if = "map_is_empty")
    )]
    pub environment: BTreeMap<String, String>,
    /// Whether the root filesystem is read-only.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub read_only_rootfs: Option<bool>,
    /// Whether a temporary filesystem is mounted at `/tmp`.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub tmpfs_tmp: Option<bool>,
    /// Optional seccomp profile.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub seccomp_profile: Option<String>,
    /// Optional working directory.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub work_dir: Option<String>,
    /// Optional Firejail profile.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub firejail_profile: Option<String>,
    /// Optional runtime path.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub runtime_path: Option<String>,
}

/// NanoVMS VM configuration.
///
/// The upstream Go type currently has no JSON tags, so its nested keys retain
/// Go's default exported names. This shape is included for completeness but is
/// not required by the minimal fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoVmConfig {
    /// VM configuration name.
    #[cfg_attr(feature = "serde", serde(rename = "Name"))]
    pub name: String,
    /// VM flavor.
    #[cfg_attr(feature = "serde", serde(rename = "VMFlavor"))]
    pub vm_flavor: String,
    /// VM image.
    #[cfg_attr(feature = "serde", serde(rename = "Image"))]
    pub image: String,
    /// Resource limits.
    #[cfg_attr(feature = "serde", serde(rename = "Resources"))]
    pub resources: NanoResourceConfig,
    /// Network configuration.
    #[cfg_attr(feature = "serde", serde(rename = "Network"))]
    pub network: NanoNetworkConfig,
}

/// NanoVMS native sandbox configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoNativeSandboxConfig {
    /// Native isolation implementation.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: String,
    /// Optional command.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "vec_is_empty"))]
    pub command: Vec<String>,
    /// Optional working directory.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub work_dir: Option<String>,
    /// Native environment values.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "env", default, skip_serializing_if = "map_is_empty")
    )]
    pub environment: BTreeMap<String, String>,
    /// Read-only native sandbox.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub read_only: Option<bool>,
    /// Whether networking is enabled.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub network: Option<bool>,
    /// Native resource limits.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub resources: Option<NanoResourceConfig>,
}

/// NanoVMS resource limits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoResourceConfig {
    /// Number of CPUs.
    pub cpu: i32,
    /// Memory limit in megabytes.
    pub memory: i32,
    /// Disk limit in megabytes.
    pub disk: i32,
}

/// NanoVMS network configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoNetworkConfig {
    /// Network type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: String,
    /// Network subnet.
    pub subnet: String,
    /// Port mappings.
    pub ports: Vec<NanoPortMapping>,
}

/// NanoVMS port mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoPortMapping {
    /// Host port.
    pub host_port: i32,
    /// Sandbox/container port.
    pub container_port: i32,
    /// Transport protocol.
    pub protocol: String,
}

/// NanoVMS filesystem mount.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NanoMount {
    /// Host source path.
    pub source: String,
    /// Sandbox target path.
    pub target: String,
    /// Mount type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: String,
    /// Whether the mount is read-only.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub read_only: bool,
}

#[cfg(feature = "serde")]
fn map_is_empty<T>(value: &BTreeMap<String, T>) -> bool {
    value.is_empty()
}

#[cfg(feature = "serde")]
fn vec_is_empty<T>(value: &[T]) -> bool {
    value.is_empty()
}

#[cfg(feature = "serde")]
fn is_false(value: &bool) -> bool {
    !*value
}

impl RenderedPlan {
    /// Verify that the content still matches the advertised digest.
    pub fn verify_digest(&self) -> bool {
        self.digest == format!("sha256:{:x}", Sha256::digest(self.content.as_bytes()))
    }
    /// Convert a non-NanoVMS plan into a BytePort apply handoff.
    pub fn byteport_handoff(&self) -> Result<BytePortHandoff, CompositionError> {
        if !self.verify_digest() {
            return Err(CompositionError::Invalid(
                "rendered plan digest does not match content".into(),
            ));
        }
        if self.target == Target::NanoVms {
            return Err(CompositionError::Invalid(
                "NanoVMS plans must use the execution handoff".into(),
            ));
        }
        Ok(BytePortHandoff {
            composition_name: self.composition_name.clone(),
            target: self.target,
            digest: self.digest.clone(),
            content: self.content.clone(),
        })
    }
    /// Convert a NanoVMS plan into an execution handoff.
    pub fn nanovms_handoff(&self) -> Result<NanoVmsHandoff, CompositionError> {
        if !self.verify_digest() {
            return Err(CompositionError::Invalid(
                "rendered plan digest does not match content".into(),
            ));
        }
        if self.target != Target::NanoVms {
            return Err(CompositionError::Invalid(
                "only NanoVMS plans can use the execution handoff".into(),
            ));
        }
        Ok(NanoVmsHandoff {
            composition_name: self.composition_name.clone(),
            digest: self.digest.clone(),
            content: self.content.clone(),
        })
    }
    /// Create a runtime handoff after checking backend/renderer compatibility.
    pub fn execution_handoff(&self, backend: ExecutionBackend) -> Result<ExecutionHandoff, CompositionError> {
        if !self.verify_digest() {
            return Err(CompositionError::Invalid(
                "rendered plan digest does not match content".into(),
            ));
        }
        if !backend.supports(self.target) {
            return Err(CompositionError::Invalid(format!(
                "backend {backend:?} cannot consume target {:?}",
                self.target
            )));
        }
        Ok(ExecutionHandoff {
            backend,
            composition_name: self.composition_name.clone(),
            digest: self.digest.clone(),
            content: self.content.clone(),
        })
    }
    /// Build an owner-scoped desired-state intent for BytePort.
    pub fn mesh_intent(
        &self,
        owner: impl Into<String>,
        artifact_ref: impl Into<String>,
        backend: ExecutionBackend,
    ) -> Result<MeshWorkloadIntent, CompositionError> {
        if !self.verify_digest() {
            return Err(CompositionError::Invalid(
                "rendered plan digest does not match content".into(),
            ));
        }
        if !backend.supports(self.target) {
            return Err(CompositionError::Invalid(format!(
                "backend {backend:?} cannot consume target {:?}",
                self.target
            )));
        }
        let owner = owner.into();
        let artifact_ref = artifact_ref.into();
        if owner.trim().is_empty() {
            return Err(CompositionError::Invalid("mesh intent owner is required".into()));
        }
        if artifact_ref.trim().is_empty() {
            return Err(CompositionError::Invalid("mesh intent artifact_ref is required".into()));
        }
        Ok(MeshWorkloadIntent {
            owner,
            composition_name: self.composition_name.clone(),
            digest: self.digest.clone(),
            artifact_ref,
            backend,
        })
    }
}

/// Composition validation or rendering error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompositionError {
    /// Invalid field or relationship.
    #[error("invalid composition: {0}")]
    Invalid(String),
    /// Requested target was not declared.
    #[error("target {0:?} was not declared")]
    TargetNotDeclared(Target),
}

impl Composition {
    /// Validate names, executable inputs, ports, dependencies, and cycles.
    pub fn validate(&self) -> Result<(), CompositionError> {
        valid_name(&self.name)?;
        for (name, service) in &self.services {
            valid_name(name)?;
            if service.image.as_deref().unwrap_or("").is_empty() && service.command.as_ref().map_or(true, Vec::is_empty)
            {
                return Err(CompositionError::Invalid(format!(
                    "service {name} needs image or command"
                )));
            }
            let mut names = BTreeSet::new();
            for port in &service.ports {
                if port.container_port == 0 || !names.insert(port.name.clone()) {
                    return Err(CompositionError::Invalid(format!(
                        "invalid or duplicate port on {name}"
                    )));
                }
            }
            for dep in &service.depends_on {
                if !self.services.contains_key(dep) {
                    return Err(CompositionError::Invalid(format!(
                        "service {name} depends on unknown {dep}"
                    )));
                }
            }
        }
        for name in self.services.keys() {
            let mut seen = BTreeSet::new();
            self.visit(name, &mut seen, &mut BTreeSet::new())?;
        }
        Ok(())
    }
    fn visit(
        &self,
        name: &str,
        path: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> Result<(), CompositionError> {
        if done.contains(name) {
            return Ok(());
        }
        if !path.insert(name.to_string()) {
            return Err(CompositionError::Invalid(format!("dependency cycle at {name}")));
        }
        for dep in &self.services[name].depends_on {
            self.visit(dep, path, done)?;
        }
        path.remove(name);
        done.insert(name.to_string());
        Ok(())
    }
    /// Render one declared target deterministically.
    pub fn render(&self, target: Target) -> Result<RenderedPlan, CompositionError> {
        self.validate()?;
        if !self.targets.contains(&target) {
            return Err(CompositionError::TargetNotDeclared(target));
        }
        let content = render_text(self, target);
        let digest = format!("sha256:{:x}", Sha256::digest(content.as_bytes()));
        Ok(RenderedPlan {
            target,
            composition_name: self.name.clone(),
            digest,
            content_type: target.content_type(),
            content,
        })
    }
}

fn protocol_name(protocol: Protocol) -> &'static str {
    match protocol {
        Protocol::Tcp => "tcp",
        Protocol::Udp => "udp",
    }
}

fn valid_name(name: &str) -> Result<(), CompositionError> {
    if name.is_empty()
        || name.len() > 63
        || name.starts_with('-')
        || name.ends_with('-')
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(CompositionError::Invalid(format!(
            "name {name:?} is not DNS-compatible"
        )));
    }
    Ok(())
}
fn yaml_quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn shell_quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn render_text(c: &Composition, target: Target) -> String {
    let mut out = String::new();
    match target {
        Target::Docker => {
            out.push_str("services:\n");
            for (name, service) in &c.services {
                out.push_str(&format!("  {name}:\n"));
                if let Some(image) = &service.image {
                    out.push_str(&format!("    image: {}\n", yaml_quote(image)));
                }
                if let Some(command) = &service.command {
                    out.push_str(&format!(
                        "    command: [{}]\n",
                        command.iter().map(|x| yaml_quote(x)).collect::<Vec<_>>().join(", ")
                    ));
                }
                if !service.environment.is_empty() {
                    out.push_str("    environment:\n");
                    for (key, value) in &service.environment {
                        out.push_str(&format!("      {}: {}\n", yaml_quote(key), yaml_quote(value)));
                    }
                }
                if !service.ports.is_empty() {
                    out.push_str("    ports:\n");
                    for port in &service.ports {
                        out.push_str(&format!(
                            "      - {}\n",
                            yaml_quote(&format!("{}/{}", port.container_port, protocol_name(port.protocol)))
                        ));
                    }
                }
                if !service.depends_on.is_empty() {
                    out.push_str("    depends_on:\n");
                    for dependency in &service.depends_on {
                        out.push_str(&format!("      - {}\n", yaml_quote(dependency)));
                    }
                }
                if let Some(h) = &service.health {
                    out.push_str("    healthcheck:\n      test: [\"CMD-SHELL\", ");
                    out.push_str(&yaml_quote(match h.kind {
                        HealthKind::Http => "curl",
                        HealthKind::Tcp => "nc",
                        HealthKind::Command => "test",
                    }));
                    out.push_str("]\n");
                }
                if service.resources.cpu.is_some() || service.resources.memory.is_some() {
                    out.push_str("    deploy:\n      resources:\n        limits:\n");
                    if let Some(cpu) = &service.resources.cpu {
                        out.push_str(&format!("          cpus: {}\n", yaml_quote(cpu)));
                    }
                    if let Some(mem) = &service.resources.memory {
                        out.push_str(&format!("          memory: {}\n", yaml_quote(mem)));
                    }
                }
            }
        }
        Target::Kubernetes => {
            for (name, service) in &c.services {
                out.push_str(&format!(
                    "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {name}\n  namespace: {}\nspec:\n  selector:\n    matchLabels:\n      app: {name}\n  template:\n    metadata:\n      labels:\n        app: {name}\n    spec:\n      containers:\n      - name: {name}\n        image: {}\n",
                    yaml_quote(&c.name),
                    yaml_quote(service.image.as_deref().unwrap_or("scratch"))
                ));
                if let Some(cmd) = &service.command {
                    out.push_str(&format!(
                        "        command: [{}]\n",
                        cmd.iter().map(|x| yaml_quote(x)).collect::<Vec<_>>().join(", ")
                    ));
                }
                if !service.environment.is_empty() {
                    out.push_str("        env:\n");
                    for (k, v) in &service.environment {
                        out.push_str(&format!(
                            "        - name: {}\n          value: {}\n",
                            yaml_quote(k),
                            yaml_quote(v)
                        ));
                    }
                }
                if !service.ports.is_empty() {
                    out.push_str("        ports:\n");
                    for p in &service.ports {
                        out.push_str(&format!(
                            "        - name: {}\n          containerPort: {}\n          protocol: {}\n",
                            yaml_quote(&p.name),
                            p.container_port,
                            protocol_name(p.protocol).to_ascii_uppercase()
                        ));
                    }
                }
                if service.resources.cpu.is_some() || service.resources.memory.is_some() {
                    out.push_str("        resources:\n          limits:\n");
                    if let Some(v) = &service.resources.cpu {
                        out.push_str(&format!("            cpu: {}\n", yaml_quote(v)));
                    }
                    if let Some(v) = &service.resources.memory {
                        out.push_str(&format!("            memory: {}\n", yaml_quote(v)));
                    }
                }
                if let Some(h) = &service.health {
                    if h.kind == HealthKind::Http {
                        out.push_str(&format!(
                            "        livenessProbe:\n          httpGet:\n            path: {}\n            port: {}\n",
                            yaml_quote(h.path.as_deref().unwrap_or("/")),
                            h.port.unwrap_or(80)
                        ));
                    }
                }
                out.push_str("---\n");
                if !service.ports.is_empty() {
                    out.push_str(&format!(
                        "apiVersion: v1\nkind: Service\nmetadata:\n  name: {name}\n  namespace: {}\nspec:\n  selector:\n    app: {name}\n  ports:\n",
                        yaml_quote(&c.name)
                    ));
                    for p in &service.ports {
                        out.push_str(&format!(
                            "  - name: {}\n    port: {}\n    targetPort: {}\n    protocol: {}\n",
                            yaml_quote(&p.name),
                            p.container_port,
                            p.container_port,
                            protocol_name(p.protocol).to_ascii_uppercase()
                        ));
                    }
                    out.push_str("---\n");
                }
            }
        }
        Target::Process => {
            out.push_str("version: \"0.5\"\nservices:\n");
            for (name, service) in &c.services {
                let command = service
                    .command
                    .as_ref()
                    .map(|v| v.iter().map(|arg| shell_quote(arg)).collect::<Vec<_>>().join(" "))
                    .or_else(|| service.image.as_ref().map(|image| shell_quote(image)))
                    .unwrap_or_default();
                out.push_str(&format!("  {name}:\n    command: {command}\n"));
                if !service.environment.is_empty() {
                    out.push_str("    environment:\n");
                    for (k, v) in &service.environment {
                        out.push_str(&format!("      - {}\n", yaml_quote(&format!("{k}={v}"))));
                    }
                }
                if !service.depends_on.is_empty() {
                    out.push_str("    depends_on:\n");
                    for d in &service.depends_on {
                        out.push_str(&format!("      - {}\n", yaml_quote(d)));
                    }
                }
                if let Some(h) = &service.health {
                    let key = match h.kind {
                        HealthKind::Http => "http_get",
                        HealthKind::Tcp => "tcp",
                        HealthKind::Command => "command",
                    };
                    let endpoint = match h.kind {
                        HealthKind::Http => format!(
                            "http://localhost:{}{}",
                            h.port.unwrap_or(80),
                            h.path.as_deref().unwrap_or("/")
                        ),
                        HealthKind::Tcp => format!("localhost:{}", h.port.unwrap_or(80)),
                        HealthKind::Command => h.path.clone().unwrap_or_default(),
                    };
                    out.push_str("    health_check:\n");
                    out.push_str(&format!("      {key}: {}\n", yaml_quote(&endpoint)));
                    out.push_str("    readiness_probe:\n");
                    out.push_str(&format!("      {key}: {}\n", yaml_quote(&endpoint)));
                }
            }
        }
        Target::NanoVms => {
            out.push_str(&format!("{{\"name\":{},\"services\":[", yaml_quote(&c.name)));
            for (i, (name, service)) in c.services.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"name\":{},\"image\":{},\"command\":{},\"cpu\":{},\"memory\":{},\"health\":{},\"depends_on\":[{}]}}",
                    yaml_quote(name),
                    yaml_quote(service.image.as_deref().unwrap_or("")),
                    yaml_quote(&service.command.as_ref().map(|v| v.join(" ")).unwrap_or_default()),
                    yaml_quote(service.resources.cpu.as_deref().unwrap_or("")),
                    yaml_quote(service.resources.memory.as_deref().unwrap_or("")),
                    yaml_quote(&service.health.as_ref().and_then(|h| h.path.clone()).unwrap_or_default()),
                    service
                        .depends_on
                        .iter()
                        .map(|dependency| yaml_quote(dependency))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            out.push_str("]}\n");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Composition {
        let mut services = BTreeMap::new();
        services.insert(
            "web".into(),
            Service {
                image: Some("nginx:1".into()),
                command: None,
                environment: BTreeMap::new(),
                ports: vec![Port {
                    name: "http".into(),
                    container_port: 80,
                    protocol: Protocol::Tcp,
                }],
                health: None,
                resources: Resources::default(),
                depends_on: BTreeSet::new(),
            },
        );
        Composition {
            name: "demo".into(),
            services,
            targets: [Target::Docker, Target::Kubernetes, Target::Process, Target::NanoVms]
                .into_iter()
                .collect(),
        }
    }
    #[test]
    fn all_targets_render_and_digest_is_stable() {
        let c = sample();
        for t in c.targets.clone() {
            let a = c.render(t).unwrap();
            let b = c.render(t).unwrap();
            assert_eq!(a, b);
            assert!(a.digest.starts_with("sha256:"));
            assert!(a.verify_digest());
        }
    }
    #[test]
    fn detects_unknown_dependency() {
        let mut c = sample();
        c.services.get_mut("web").unwrap().depends_on.insert("db".into());
        assert!(c.validate().is_err());
    }
    #[test]
    fn handoffs_enforce_target_ownership() {
        let c = sample();
        let docker = c.render(Target::Docker).unwrap();
        assert!(docker.byteport_handoff().is_ok());
        assert!(docker.nanovms_handoff().is_err());
        let nvms = c.render(Target::NanoVms).unwrap();
        assert!(nvms.nanovms_handoff().is_ok());
        assert!(nvms.byteport_handoff().is_err());
    }
    #[test]
    fn execution_backends_match_plan_formats() {
        let c = sample();
        let docker = c.render(Target::Docker).unwrap();
        for backend in [
            ExecutionBackend::Podman,
            ExecutionBackend::AppleContainers,
            ExecutionBackend::WslContainers,
        ] {
            assert_eq!(docker.execution_handoff(backend).unwrap().backend, backend);
        }
        assert!(docker.execution_handoff(ExecutionBackend::NanoVms).is_err());
        let nvms = c.render(Target::NanoVms).unwrap();
        assert!(nvms.execution_handoff(ExecutionBackend::NanoVms).is_ok());
    }
    #[test]
    fn mesh_intent_is_owner_scoped() {
        let c = sample();
        let plan = c.render(Target::Docker).unwrap();
        let intent = plan
            .mesh_intent("alice", "oci://registry/demo@sha256:abc", ExecutionBackend::Podman)
            .unwrap();
        assert_eq!(intent.owner, "alice");
        assert_eq!(intent.digest, plan.digest);
        assert!(plan.mesh_intent("", "oci://x", ExecutionBackend::Podman).is_err());
    }
    #[test]
    fn renderer_preserves_docker_fields_and_escapes_values() {
        let mut c = sample();
        let service = c.services.get_mut("web").unwrap();
        service.image = Some("evil\nimage".into());
        service.environment.insert("TOKEN".into(), "line\nvalue".into());
        let docker = c.render(Target::Docker).unwrap();
        assert!(docker.content.contains("environment:"));
        assert!(docker.content.contains("\"TOKEN\": \"line\\nvalue\""));
        assert!(docker.content.contains("ports:"));
        assert!(!docker.content.contains("evil\nimage"));
        assert!(docker.content.contains("evil\\nimage"));
        let process = c.render(Target::Process).unwrap();
        assert!(!process.content.contains("evil\nimage"));
    }

    #[test]
    fn target_renderers_match_checked_in_schemas() {
        let mut c = sample();
        let service = c.services.get_mut("web").unwrap();
        service.command = Some(vec!["serve".into(), "--port".into(), "80".into()]);
        service.environment.insert("MODE".into(), "test".into());
        service.health = Some(HealthCheck {
            kind: HealthKind::Http,
            path: Some("/healthz".into()),
            port: Some(80),
        });

        let kubernetes = c.render(Target::Kubernetes).unwrap().content;
        assert!(kubernetes.contains("  namespace: \"demo\""));
        assert!(kubernetes.contains("  selector:\n    matchLabels:\n      app: web"));
        assert!(kubernetes.contains("      labels:\n        app: web"));
        assert!(kubernetes.contains("kind: Service"));
        assert!(kubernetes.contains("protocol: TCP"));

        let process = c.render(Target::Process).unwrap().content;
        assert!(process.starts_with("version: \"0.5\"\nservices:\n"));
        assert!(process.contains("      - \"MODE=test\""));
        assert!(process.contains("    readiness_probe:\n      http_get: \"http://localhost:80/healthz\""));

        let nanovms = c.render(Target::NanoVms).unwrap().content;
        assert!(nanovms.contains("\"depends_on\":[]"));
    }

    #[test]
    fn nanovms_dependencies_are_deterministic() {
        let mut c = sample();
        let mut db = c.services.get("web").unwrap().clone();
        db.image = Some("postgres:16".into());
        c.services.insert("db".into(), db);
        c.services.get_mut("web").unwrap().depends_on.insert("db".into());
        let first = c.render(Target::NanoVms).unwrap().content;
        let second = c.render(Target::NanoVms).unwrap().content;
        assert_eq!(first, second);
        assert!(first.contains("\"depends_on\":[\"db\"]"));
    }

    #[test]
    fn tampering_invalidates_handoff_digest() {
        let c = sample();
        let mut plan = c.render(Target::NanoVms).unwrap();
        plan.content.push('x');
        assert!(!plan.verify_digest());
        assert!(plan.nanovms_handoff().is_err());
        assert!(plan.execution_handoff(ExecutionBackend::NanoVms).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn bridge_fixture_round_trips_deterministically() {
        let fixture = include_str!("../../../integrations/foundation-pilot/bridge.json");
        let bridge: BridgeContract = serde_json::from_str(fixture).unwrap();
        let rendered = format!("{}\n", serde_json::to_string_pretty(&bridge).unwrap());
        assert_eq!(rendered, fixture);
        assert_eq!(bridge.byteport.execution_backend, BytePortExecutionBackend::NanoVms);
        assert_eq!(bridge.nanovms.len(), 1);
        assert_eq!(
            bridge.nanovms[0].environment.get("phenocompose.sha256"),
            Some(&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned())
        );
    }
}
