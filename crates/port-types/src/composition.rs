//! A small, deterministic composition model shared by renderers.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
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
                out.push_str(&format!("apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {name}\nspec:\n  template:\n    spec:\n      containers:\n      - name: {name}\n        image: {}\n", yaml_quote(service.image.as_deref().unwrap_or("scratch"))));
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
                        out.push_str(&format!("        - containerPort: {}\n", p.container_port));
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
            }
        }
        Target::Process => {
            out.push_str("version: \"0.5\"\nprocesses:\n");
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
                        out.push_str(&format!("      {}: {}\n", yaml_quote(k), yaml_quote(v)));
                    }
                }
                if !service.depends_on.is_empty() {
                    out.push_str("    depends_on:\n");
                    for d in &service.depends_on {
                        out.push_str(&format!("      - {}\n", yaml_quote(d)));
                    }
                }
                if let Some(h) = &service.health {
                    out.push_str(&format!(
                        "    readiness_probe: {}\n",
                        yaml_quote(h.path.as_deref().unwrap_or("/"))
                    ));
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
                    "{{\"name\":{},\"image\":{},\"command\":{},\"cpu\":{},\"memory\":{},\"health\":{}}}",
                    yaml_quote(name),
                    yaml_quote(service.image.as_deref().unwrap_or("")),
                    yaml_quote(&service.command.as_ref().map(|v| v.join(" ")).unwrap_or_default()),
                    yaml_quote(service.resources.cpu.as_deref().unwrap_or("")),
                    yaml_quote(service.resources.memory.as_deref().unwrap_or("")),
                    yaml_quote(&service.health.as_ref().and_then(|h| h.path.clone()).unwrap_or_default())
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
    fn tampering_invalidates_handoff_digest() {
        let c = sample();
        let mut plan = c.render(Target::NanoVms).unwrap();
        plan.content.push('x');
        assert!(!plan.verify_digest());
        assert!(plan.nanovms_handoff().is_err());
        assert!(plan.execution_handoff(ExecutionBackend::NanoVms).is_err());
    }
}
