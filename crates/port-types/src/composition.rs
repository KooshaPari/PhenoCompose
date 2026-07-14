//! A small, deterministic composition model shared by renderers.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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

impl Target { fn content_type(self) -> &'static str { match self { Self::Docker => "application/vnd.docker.compose+yaml", Self::Kubernetes => "application/vnd.kubernetes+yaml", Self::Process => "application/vnd.process-compose+yaml", Self::NanoVms => "application/vnd.nanovms.plan+json" } } }

/// A service in a composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service { /// Optional OCI image.
    pub image: Option<String>, /// Optional executable and arguments.
    pub command: Option<Vec<String>>, /// Environment values; secret values must be references.
    pub environment: BTreeMap<String, String>, /// Published ports.
    pub ports: Vec<Port>, /// Optional health check.
    pub health: Option<HealthCheck>, /// Resource hints.
    pub resources: Resources, /// Startup dependencies.
    pub depends_on: BTreeSet<String>, }

/// A service port.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Port { /// Stable port name.
    pub name: String, /// Container port.
    pub container_port: u16, /// Transport protocol.
    pub protocol: Protocol, }

/// Port transport protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol { /// TCP.
    Tcp, /// UDP.
    Udp }

/// Health check declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheck { /// Check kind.
    pub kind: HealthKind, /// Optional endpoint path for HTTP checks.
    pub path: Option<String>, /// Optional port.
    pub port: Option<u16> }

/// Health check kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthKind { /// HTTP GET.
    Http, /// TCP connect.
    Tcp, /// Process command.
    Command }

/// Resource hints passed to a target renderer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resources { /// CPU request.
    pub cpu: Option<String>, /// Memory request.
    pub memory: Option<String> }

/// A complete composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition { /// DNS-compatible composition name.
    pub name: String, /// Services keyed by stable name.
    pub services: BTreeMap<String, Service>, /// Requested render targets.
    pub targets: BTreeSet<Target> }

/// A rendered target artifact with a content digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPlan { /// Target used.
    pub target: Target, /// Composition name.
    pub composition_name: String, /// SHA-256 digest of content.
    pub digest: String, /// MIME-like content type.
    pub content_type: &'static str, /// Rendered bytes as UTF-8.
    pub content: String }

/// Composition validation or rendering error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompositionError { /// Invalid field or relationship.
    #[error("invalid composition: {0}")] Invalid(String), /// Requested target was not declared.
    #[error("target {0:?} was not declared")]
    TargetNotDeclared(Target) }

impl Composition {
    /// Validate names, executable inputs, ports, dependencies, and cycles.
    pub fn validate(&self) -> Result<(), CompositionError> {
        valid_name(&self.name)?;
        for (name, service) in &self.services {
            valid_name(name)?;
            if service.image.as_deref().unwrap_or("").is_empty() && service.command.as_ref().map_or(true, Vec::is_empty) { return Err(CompositionError::Invalid(format!("service {name} needs image or command"))); }
            let mut names = BTreeSet::new();
            for port in &service.ports { if port.container_port == 0 || !names.insert(port.name.clone()) { return Err(CompositionError::Invalid(format!("invalid or duplicate port on {name}"))); } }
            for dep in &service.depends_on { if !self.services.contains_key(dep) { return Err(CompositionError::Invalid(format!("service {name} depends on unknown {dep}"))); } }
        }
        for name in self.services.keys() { let mut seen = BTreeSet::new(); self.visit(name, &mut seen, &mut BTreeSet::new())?; }
        Ok(())
    }
    fn visit(&self, name: &str, path: &mut BTreeSet<String>, done: &mut BTreeSet<String>) -> Result<(), CompositionError> { if done.contains(name) { return Ok(()); } if !path.insert(name.to_string()) { return Err(CompositionError::Invalid(format!("dependency cycle at {name}"))); } for dep in &self.services[name].depends_on { self.visit(dep, path, done)?; } path.remove(name); done.insert(name.to_string()); Ok(()) }
    /// Render one declared target deterministically.
    pub fn render(&self, target: Target) -> Result<RenderedPlan, CompositionError> { self.validate()?; if !self.targets.contains(&target) { return Err(CompositionError::TargetNotDeclared(target)); } let content = render_text(self, target); let digest = format!("sha256:{:x}", Sha256::digest(content.as_bytes())); Ok(RenderedPlan { target, composition_name: self.name.clone(), digest, content_type: target.content_type(), content }) }
}

fn valid_name(name: &str) -> Result<(), CompositionError> { if name.is_empty() || name.len() > 63 || name.starts_with('-') || name.ends_with('-') || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') { return Err(CompositionError::Invalid(format!("name {name:?} is not DNS-compatible"))); } Ok(()) }
fn render_text(c: &Composition, target: Target) -> String {
    let mut out = String::new();
    match target {
        Target::Docker => {
            out.push_str("services:\n");
            for (name, service) in &c.services {
                out.push_str(&format!("  {name}:\n"));
                if let Some(image) = &service.image { out.push_str(&format!("    image: {image}\n")); }
                if let Some(command) = &service.command { out.push_str(&format!("    command: [{}]\n", command.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", "))); }
            }
        }
        Target::Kubernetes => for (name, service) in &c.services {
            out.push_str(&format!("apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {name}\nspec:\n  template:\n    spec:\n      containers:\n      - name: {name}\n        image: {}\n---\n", service.image.as_deref().unwrap_or("scratch")));
        }
        Target::Process => {
            out.push_str("version: \"0.5\"\nprocesses:\n");
            for (name, service) in &c.services { let command = service.command.as_ref().map(|v| v.join(" ")).or_else(|| service.image.clone()).unwrap_or_default(); out.push_str(&format!("  {name}:\n    command: {command}\n")); }
        }
        Target::NanoVms => {
            out.push_str(&format!("{{\"name\":\"{}\",\"services\":[", c.name));
            for (i, (name, service)) in c.services.iter().enumerate() { if i > 0 { out.push(','); } out.push_str(&format!("{{\"name\":\"{name}\",\"image\":{:?}}}", service.image.as_deref().unwrap_or(""))); }
            out.push_str("]}\n");
        }
    }
    out
}

#[cfg(test)]
mod tests { use super::*; fn sample() -> Composition { let mut services=BTreeMap::new(); services.insert("web".into(), Service { image:Some("nginx:1".into()), command:None, environment:BTreeMap::new(), ports:vec![Port{name:"http".into(),container_port:80,protocol:Protocol::Tcp}], health:None, resources:Resources::default(), depends_on:BTreeSet::new() }); Composition{name:"demo".into(),services,targets:[Target::Docker,Target::Kubernetes,Target::Process,Target::NanoVms].into_iter().collect()} } #[test] fn all_targets_render_and_digest_is_stable(){ let c=sample(); for t in c.targets.clone(){ let a=c.render(t).unwrap(); let b=c.render(t).unwrap(); assert_eq!(a,b); assert!(a.digest.starts_with("sha256:")); } } #[test] fn detects_unknown_dependency(){ let mut c=sample(); c.services.get_mut("web").unwrap().depends_on.insert("db".into()); assert!(c.validate().is_err()); } }
