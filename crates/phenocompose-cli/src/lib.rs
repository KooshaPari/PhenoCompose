#![forbid(unsafe_code)]

pub mod model;

use model::{CompositionManifest, GpuVendor, Plan, ProviderStatus, RuntimeProvider, Service};
use phenocompose_port_composer::{ComposeError, Composer};
use phenocompose_port_publisher::{PublishError, Publisher};
use phenocompose_port_runtime::{Runtime, RuntimeError};
use phenocompose_port_types::{
    ComposedArtifact, ContainerId, ContainerStatus, ImageRef, Manifest, PublishReceipt, PublishTarget,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct CliError {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub capability: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Validation,
    Unsupported,
    NotFound,
    Conflict,
    Backend,
    Io,
}

#[derive(Serialize)]
pub struct ErrorEnvelope<'a> {
    pub error: ErrorBody<'a>,
}

#[derive(Serialize)]
pub struct ErrorBody<'a> {
    pub kind: ErrorKind,
    pub code: &'a str,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<&'a str>,
}

impl CliError {
    pub fn validation(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, code, message)
    }

    pub fn unsupported(
        code: impl Into<String>,
        message: impl Into<String>,
        capability: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            kind: ErrorKind::Unsupported,
            code: code.into(),
            message: message.into(),
            capability: Some(capability.into()),
            provider: Some(provider.into()),
        }
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, code, message)
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, code, message)
    }

    pub fn backend(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Backend, code, message)
    }

    pub fn io(code: impl Into<String>, error: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, code, error.to_string())
    }

    pub fn json(error: serde_json::Error) -> Self {
        Self::validation("json", error.to_string())
    }

    fn new(kind: ErrorKind, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            capability: None,
            provider: None,
        }
    }

    pub fn envelope(&self) -> ErrorEnvelope<'_> {
        ErrorEnvelope {
            error: ErrorBody {
                kind: self.kind,
                code: &self.code,
                message: &self.message,
                capability: self.capability.as_deref(),
                provider: self.provider.as_deref(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunState {
    pub state_version: String,
    pub run_id: String,
    pub manifest_sha256: String,
    pub provider: String,
    pub created_unix_seconds: u64,
    pub lifecycle: RunLifecycle,
    pub containers: BTreeMap<String, String>,
    pub manifest: CompositionManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunLifecycle {
    Running,
    Down,
}

#[derive(Debug, Serialize)]
pub struct ApplyOutput {
    pub run_id: String,
    pub manifest_sha256: String,
    pub provider: String,
    pub dry_run: bool,
    pub mutation: bool,
    pub containers: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct StatusOutput {
    pub run_id: String,
    pub lifecycle: RunLifecycle,
    pub services: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct Provenance {
    pub provenance_version: String,
    pub run_id: String,
    pub manifest_sha256: String,
    pub effective_engine: String,
    pub created_unix_seconds: u64,
    pub containers: BTreeMap<String, String>,
}

pub fn load_manifest(path: &Path) -> Result<CompositionManifest> {
    let input = fs::read_to_string(path).map_err(|error| CliError::io("manifest_read", error))?;
    CompositionManifest::parse(&input)
}

pub fn render_plan(manifest: &CompositionManifest) -> Result<Plan> {
    manifest.plan()
}

pub fn apply(manifest: CompositionManifest, state_dir: &Path, dry_run: bool) -> Result<ApplyOutput> {
    let plan = manifest.plan()?;
    let run_id = format!(
        "{}-{}",
        sanitize_name(&manifest.metadata.name),
        &plan.manifest_sha256[..12]
    );
    if dry_run {
        return Ok(ApplyOutput {
            run_id,
            manifest_sha256: plan.manifest_sha256,
            provider: runtime_provider_name(&manifest.runtime.provider).to_owned(),
            dry_run: true,
            mutation: false,
            containers: BTreeMap::new(),
        });
    }
    ensure_apply_capabilities(&manifest)?;
    let state_path = state_path(state_dir, &run_id);
    if state_path.exists() {
        return Err(CliError::conflict(
            "run_exists",
            format!("run {run_id} already has persisted state"),
        ));
    }

    let mut containers = BTreeMap::new();
    let order = manifest.service_order()?;
    let composer = PrebuiltImageComposer;
    let publisher = PodmanLocalPublisher::new(&manifest.runtime.wsl_distribution);
    publisher.probe()?;

    for name in order {
        let service = &manifest.services[&name];
        let port_manifest = Manifest::new(&name).with_tag("image", &service.image);
        let artifact = composer.compose(&port_manifest).map_err(map_compose_error)?;
        publisher
            .publish(&artifact, &PublishTarget::new("podman-local", &service.image))
            .map_err(map_publish_error)?;
        let runtime = PodmanRuntime::for_service(&manifest.runtime.wsl_distribution, &run_id, &name, service);
        match runtime.spawn(&artifact.image) {
            Ok(container_id) => {
                containers.insert(name, container_id.id);
            }
            Err(error) => {
                rollback(&manifest.runtime.wsl_distribution, &containers);
                return Err(map_runtime_error(error));
            }
        }
    }

    let state = RunState {
        state_version: "phenocompose.run/v0".to_owned(),
        run_id: run_id.clone(),
        manifest_sha256: plan.manifest_sha256.clone(),
        provider: "podman".to_owned(),
        created_unix_seconds: unix_seconds()?,
        lifecycle: RunLifecycle::Running,
        containers: containers.clone(),
        manifest,
    };
    if let Err(error) = save_state(state_dir, &state) {
        rollback(&state.manifest.runtime.wsl_distribution, &containers);
        return Err(error);
    }
    Ok(ApplyOutput {
        run_id,
        manifest_sha256: plan.manifest_sha256,
        provider: "podman".to_owned(),
        dry_run: false,
        mutation: true,
        containers,
    })
}

pub fn status(state_dir: &Path, run_id: &str) -> Result<StatusOutput> {
    let state = load_state(state_dir, run_id)?;
    ensure_persisted_provider(&state)?;
    let runtime = PodmanRuntime::bare(&state.manifest.runtime.wsl_distribution);
    let mut services = BTreeMap::new();
    for (service, id) in &state.containers {
        let value = runtime
            .status(&ContainerId::new(id))
            .map_err(map_runtime_error)?
            .to_string();
        services.insert(service.clone(), value);
    }
    Ok(StatusOutput {
        run_id: state.run_id,
        lifecycle: state.lifecycle,
        services,
    })
}

pub fn down(state_dir: &Path, run_id: &str) -> Result<StatusOutput> {
    let mut state = load_state(state_dir, run_id)?;
    ensure_persisted_provider(&state)?;
    if state.lifecycle == RunLifecycle::Down {
        return Err(CliError::conflict(
            "run_already_down",
            format!("run {run_id} is already down"),
        ));
    }
    let runtime = PodmanRuntime::bare(&state.manifest.runtime.wsl_distribution);
    let mut order = if state.manifest.teardown.order.is_empty() {
        let mut order = state.manifest.service_order()?;
        order.reverse();
        order
    } else {
        state.manifest.teardown.order.clone()
    };
    for service in order.drain(..) {
        if let Some(id) = state.containers.get(&service) {
            match runtime.status(&ContainerId::new(id)).map_err(map_runtime_error)? {
                ContainerStatus::NotFound | ContainerStatus::Exited => {}
                _ => runtime.stop(&ContainerId::new(id)).map_err(map_runtime_error)?,
            }
        }
    }
    state.lifecycle = RunLifecycle::Down;
    save_state(state_dir, &state)?;
    let services = state
        .containers
        .keys()
        .map(|name| (name.clone(), "down".to_owned()))
        .collect();
    Ok(StatusOutput {
        run_id: state.run_id,
        lifecycle: state.lifecycle,
        services,
    })
}

pub fn run_action(state_dir: &Path, run_id: &str, action: &str) -> Result<()> {
    let state = load_state(state_dir, run_id)?;
    if !state.manifest.actions.contains_key(action) {
        return Err(CliError::not_found(
            "action_not_found",
            format!("run {run_id} has no action named {action}"),
        ));
    }
    Err(CliError::unsupported(
        "action_execution_unsupported",
        "the current Runtime port has no exec operation; the action was not run",
        "runtime.exec",
        &state.provider,
    ))
}

pub fn export_provenance(state_dir: &Path, run_id: &str) -> Result<Provenance> {
    let state = load_state(state_dir, run_id)?;
    if !state.manifest.provenance.required {
        return Err(CliError::unsupported(
            "provenance_not_requested",
            "the persisted manifest did not require provenance",
            "provenance.export",
            &state.provider,
        ));
    }
    Ok(Provenance {
        provenance_version: "phenocompose.provenance/v0".to_owned(),
        run_id: state.run_id,
        manifest_sha256: state.manifest_sha256,
        effective_engine: state.provider,
        created_unix_seconds: state.created_unix_seconds,
        containers: state.containers,
    })
}

fn ensure_apply_capabilities(manifest: &CompositionManifest) -> Result<()> {
    match manifest.runtime.provider {
        RuntimeProvider::Podman => {}
        RuntimeProvider::Nvms => {
            return Err(CliError::unsupported(
                "nvms_persistence_unsupported",
                "the NVMS driver cannot reattach to persisted instance IDs; apply was not attempted",
                "runtime.persistence",
                "nvms",
            ));
        }
        RuntimeProvider::Placeholder => {
            return Err(CliError::unsupported(
                "runtime_provider_placeholder",
                "the selected runtime is a capability placeholder",
                &manifest.runtime.capability,
                "placeholder",
            ));
        }
    }
    for (name, provider) in &manifest.providers {
        if provider.status != ProviderStatus::Available {
            return Err(CliError::unsupported(
                "provider_unavailable",
                format!("provider declaration {name} is not available"),
                &provider.capability,
                provider.implementation.as_deref().unwrap_or("placeholder"),
            ));
        }
    }
    if manifest.services.values().any(|service| service.health_check.is_some()) {
        return Err(CliError::unsupported(
            "health_check_unsupported",
            "health-check enforcement is not represented by the current Runtime port",
            "runtime.health_check",
            "podman",
        ));
    }
    if !manifest.artifacts.is_empty() {
        return Err(CliError::unsupported(
            "artifact_publication_unsupported",
            "declared artifact publication is not implemented in Slice 1",
            "publisher.artifacts",
            "podman-local",
        ));
    }
    if manifest.teardown.remove_volumes {
        return Err(CliError::unsupported(
            "volume_teardown_unsupported",
            "volume removal is not represented by the current Runtime port",
            "runtime.volume_teardown",
            "podman",
        ));
    }
    for service in manifest.services.values() {
        if let Some(gpu) = service.resources.as_ref().and_then(|r| r.gpu.as_ref()) {
            if gpu.vendor != GpuVendor::Nvidia {
                return Err(CliError::unsupported(
                    "gpu_vendor_unsupported",
                    "Slice 1 only maps NVIDIA UUID selectors to Podman CDI devices",
                    "runtime.gpu_uuid",
                    "podman",
                ));
            }
        }
    }
    Ok(())
}

fn runtime_provider_name(provider: &RuntimeProvider) -> &'static str {
    match provider {
        RuntimeProvider::Podman => "podman",
        RuntimeProvider::Nvms => "nvms",
        RuntimeProvider::Placeholder => "placeholder",
    }
}

fn ensure_persisted_provider(state: &RunState) -> Result<()> {
    if state.provider == "podman" {
        Ok(())
    } else {
        Err(CliError::unsupported(
            "persisted_provider_unsupported",
            format!("cannot operate on persisted provider {}", state.provider),
            "runtime.persistence",
            &state.provider,
        ))
    }
}

fn rollback(distribution: &Option<String>, containers: &BTreeMap<String, String>) {
    let runtime = PodmanRuntime::bare(distribution);
    for id in containers.values().rev() {
        let _ = runtime.stop(&ContainerId::new(id));
    }
}

fn state_path(state_dir: &Path, run_id: &str) -> PathBuf {
    state_dir.join(format!("{run_id}.json"))
}

fn load_state(state_dir: &Path, run_id: &str) -> Result<RunState> {
    let path = state_path(state_dir, run_id);
    let input = fs::read_to_string(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliError::not_found("run_not_found", format!("no persisted run state for {run_id}"))
        } else {
            CliError::io("state_read", error)
        }
    })?;
    serde_json::from_str(&input).map_err(CliError::json)
}

fn save_state(state_dir: &Path, state: &RunState) -> Result<()> {
    fs::create_dir_all(state_dir).map_err(|error| CliError::io("state_dir_create", error))?;
    let path = state_path(state_dir, &state.run_id);
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(state).map_err(CliError::json)?;
    fs::write(&temporary, bytes).map_err(|error| CliError::io("state_write", error))?;
    fs::rename(&temporary, &path).map_err(|error| CliError::io("state_commit", error))
}

fn unix_seconds() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| CliError::backend("clock", error.to_string()))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[derive(Debug)]
struct PrebuiltImageComposer;

impl Composer for PrebuiltImageComposer {
    fn compose(&self, manifest: &Manifest) -> std::result::Result<ComposedArtifact, ComposeError> {
        let image = manifest
            .tags
            .iter()
            .find(|(key, _)| key == "image")
            .map(|(_, value)| value)
            .ok_or_else(|| ComposeError::validation("prebuilt image tag is required"))?;
        Ok(ComposedArtifact::new(
            format!("{}:prebuilt", manifest.name),
            ImageRef::new(image),
        ))
    }

    fn name(&self) -> &str {
        "prebuilt-image"
    }
}

#[derive(Debug)]
struct PodmanLocalPublisher {
    distribution: Option<String>,
}

impl PodmanLocalPublisher {
    fn new(distribution: &Option<String>) -> Self {
        Self {
            distribution: distribution.clone(),
        }
    }

    fn probe(&self) -> Result<()> {
        let output = podman_output(&self.distribution, ["version", "--format", "json"])?;
        require_success("podman_unavailable", output).map(|_| ())
    }
}

impl Publisher for PodmanLocalPublisher {
    fn publish(
        &self,
        artifact: &ComposedArtifact,
        target: &PublishTarget,
    ) -> std::result::Result<PublishReceipt, PublishError> {
        if target.kind != "podman-local" || target.locator != artifact.image.as_ref() {
            return Err(PublishError::validation(
                "podman-local target must match the composed image",
            ));
        }
        let output = podman_output(&self.distribution, ["image", "exists", artifact.image.as_ref()])
            .map_err(|error| PublishError::transport(error.to_string()))?;
        if !output.status.success() {
            return Err(PublishError::transport(format!(
                "image {} is not present in Podman local storage",
                artifact.image
            )));
        }
        Ok(PublishReceipt::new(
            &artifact.id,
            target.clone(),
            format!("podman-local://{}", artifact.image),
        ))
    }

    fn name(&self) -> &str {
        "podman-local"
    }
}

#[derive(Debug)]
struct PodmanRuntime {
    distribution: Option<String>,
    container_name: Option<String>,
    command: Vec<String>,
    environment: BTreeMap<String, String>,
    cpu_millis: Option<u32>,
    memory_bytes: Option<u64>,
    gpu_uuids: Vec<String>,
}

impl PodmanRuntime {
    fn bare(distribution: &Option<String>) -> Self {
        Self {
            distribution: distribution.clone(),
            container_name: None,
            command: Vec::new(),
            environment: BTreeMap::new(),
            cpu_millis: None,
            memory_bytes: None,
            gpu_uuids: Vec::new(),
        }
    }

    fn for_service(distribution: &Option<String>, run_id: &str, service_name: &str, service: &Service) -> Self {
        let resources = service.resources.as_ref();
        Self {
            distribution: distribution.clone(),
            container_name: Some(format!("{run_id}-{service_name}")),
            command: service.command.clone(),
            environment: service.environment.clone(),
            cpu_millis: resources.and_then(|value| value.cpu_millis),
            memory_bytes: resources.and_then(|value| value.memory_bytes),
            gpu_uuids: resources
                .and_then(|value| value.gpu.as_ref())
                .map(|gpu| gpu.uuids.clone())
                .unwrap_or_default(),
        }
    }
}

impl Runtime for PodmanRuntime {
    fn spawn(&self, image: &ImageRef) -> std::result::Result<ContainerId, RuntimeError> {
        let name = self
            .container_name
            .as_ref()
            .ok_or_else(|| RuntimeError::validation("service configuration is required"))?;
        let mut arguments = vec![
            "run".to_owned(),
            "--detach".to_owned(),
            "--name".to_owned(),
            name.clone(),
        ];
        for (key, value) in &self.environment {
            arguments.extend(["--env".to_owned(), format!("{key}={value}")]);
        }
        if let Some(cpu_millis) = self.cpu_millis {
            arguments.extend(["--cpus".to_owned(), format!("{:.3}", f64::from(cpu_millis) / 1000.0)]);
        }
        if let Some(memory_bytes) = self.memory_bytes {
            arguments.extend(["--memory".to_owned(), memory_bytes.to_string()]);
        }
        for uuid in &self.gpu_uuids {
            arguments.extend(["--device".to_owned(), format!("nvidia.com/gpu={uuid}")]);
        }
        arguments.push(image.to_string());
        arguments.extend(self.command.iter().cloned());
        let output = podman_output_owned(&self.distribution, &arguments)
            .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if !output.status.success() {
            return Err(RuntimeError::backend(output_message(&output)));
        }
        let id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if id.is_empty() {
            return Err(RuntimeError::backend(
                "podman run succeeded without returning a container ID",
            ));
        }
        Ok(ContainerId::new(id))
    }

    fn stop(&self, id: &ContainerId) -> std::result::Result<(), RuntimeError> {
        let output = podman_output(&self.distribution, ["stop", id.as_ref()])
            .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(RuntimeError::backend(output_message(&output)))
        }
    }

    fn status(&self, id: &ContainerId) -> std::result::Result<ContainerStatus, RuntimeError> {
        let output = podman_output(
            &self.distribution,
            ["inspect", "--format", "{{.State.Status}}", id.as_ref()],
        )
        .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if !output.status.success() {
            let message = output_message(&output);
            if message.to_ascii_lowercase().contains("no such") {
                return Ok(ContainerStatus::NotFound);
            }
            return Err(RuntimeError::backend(message));
        }
        match String::from_utf8_lossy(&output.stdout).trim() {
            "running" => Ok(ContainerStatus::Running),
            "paused" => Ok(ContainerStatus::Paused),
            "exited" | "stopped" | "configured" | "created" => Ok(ContainerStatus::Exited),
            status => Err(RuntimeError::backend(format!("unrecognized Podman status {status:?}"))),
        }
    }

    fn name(&self) -> &str {
        "podman"
    }
}

fn podman_output<'a>(distribution: &Option<String>, arguments: impl IntoIterator<Item = &'a str>) -> Result<Output> {
    let arguments: Vec<String> = arguments.into_iter().map(str::to_owned).collect();
    podman_output_owned(distribution, &arguments)
}

fn podman_output_owned(distribution: &Option<String>, arguments: &[String]) -> Result<Output> {
    let mut command = if let Some(distribution) = distribution {
        let mut command = Command::new("wsl.exe");
        command.args(["-d", distribution, "--", "podman"]);
        command
    } else {
        Command::new("podman")
    };
    command
        .args(arguments)
        .output()
        .map_err(|error| CliError::io("podman_spawn", error))
}

fn require_success(code: &str, output: Output) -> Result<Output> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(CliError::backend(code, output_message(&output)))
    }
}

fn output_message(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr
    }
}

fn map_compose_error(error: ComposeError) -> CliError {
    CliError::backend("compose_failed", error.to_string())
}

fn map_publish_error(error: PublishError) -> CliError {
    CliError::backend("publish_failed", error.to_string())
}

fn map_runtime_error(error: RuntimeError) -> CliError {
    match error {
        RuntimeError::NotFound(message) => CliError::not_found("runtime_not_found", message),
        RuntimeError::Validation(message) => CliError::validation("runtime_validation", message),
        RuntimeError::Backend(message) => CliError::backend("runtime_backend", message),
        other => CliError::backend("runtime_unknown", other.to_string()),
    }
}
