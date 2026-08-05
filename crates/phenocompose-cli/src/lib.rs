#![forbid(unsafe_code)]

pub mod model;
pub mod nvms;
pub mod service_graph;

pub use service_graph::{
    ensure_service_lifecycle_capability, execute_lifecycle_plan, plan_lifecycle, IntentPhase, LifecyclePlan,
    LIFECYCLE_SCHEMA_VERSION, RollbackContract, SERVICE_LIFECYCLE_CAPABILITY,
};

use model::{
    canonical_gpu_uuid, CompositionManifest, EffectiveEngine, ExternalEngineToken, GpuVendor, Plan, ProviderStatus,
    RuntimeProvider, Service,
};
use nvms::{
    ArtifactRequirements, EvaluationClient, EvaluationRequest, EvaluationResult, GpuBinding, LifecycleClient,
    ProcessEvaluationClient, ProcessLifecycleClient, ResourceGpu, ResourceManifest, ServiceDefinition,
    ServiceLifecycleIntent, ServiceLifecycleRequest, ServiceLifecycleResult, ACTION_VERSION, DEFAULT_PODMAN_PIPE,
    RESOURCE_VERSION, SERVICE_LIFECYCLE_VERSION,
};
use phenocompose_port_composer::{ComposeError, Composer};
use phenocompose_port_publisher::{PublishError, Publisher};
use phenocompose_port_runtime::{Runtime, RuntimeError};
use phenocompose_port_types::{
    ComposedArtifact, ContainerId, ContainerStatus, ImageRef, Manifest, PublishReceipt, PublishTarget,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use wait_timeout::ChildExt;

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

/// A deterministic, non-mutating receipt from a local runtime capability
/// probe.
///
/// Probes only query the selected CLI (`podman info`, Apple Containers system
/// status/version, or WSLc image listing). They never start a machine, create a
/// container, or change persisted state. The digest covers the successful
/// command outputs so a receipt can be carried across the BytePort/NanoVMS
/// handoff without claiming more than the host actually reported.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityReceipt {
    /// Version of the receipt schema.
    pub schema_version: String,
    /// Provider-neutral runtime name.
    pub provider: String,
    /// Executable selected by the probe (including a WSL wrapper when used).
    pub executable: String,
    /// Exact argument vectors used by the probe, including a WSL wrapper when
    /// a distribution override was supplied.
    pub commands: Vec<Vec<String>>,
    /// Whether every read-only probe command exited successfully.
    pub ready: bool,
    /// Version discovered in structured output, when the provider reports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// SHA-256 of the ordered stdout/stderr probe outputs.
    pub output_sha256: String,
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
    pub lifecycle: LifecyclePlan,
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
    pub jobs: BTreeMap<String, JobProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JobProvenance {
    pub provenance_version: String,
    pub run_id: String,
    pub job_id: String,
    pub manifest_sha256: String,
    pub effective_engine: String,
    pub resolved_provider: String,
    pub execution_plane: String,
    pub action: String,
    pub service: String,
    pub command: Vec<String>,
    pub image: String,
    pub dependency_services: Vec<String>,
    pub gpu_bindings: Vec<GpuBinding>,
    pub timeout_millis: i64,
    pub max_output_bytes: usize,
    #[serde(default)]
    pub output_root: String,
    #[serde(default)]
    pub output_root_created: bool,
    #[serde(default)]
    pub output_root_available_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolkit_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolkit_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolkit_executable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<nvms::Lifecycle>,
    pub success: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error_code: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error_message: String,
}

pub fn load_manifest(path: &Path) -> Result<CompositionManifest> {
    let input = fs::read_to_string(path).map_err(|error| CliError::io("manifest_read", error))?;
    CompositionManifest::parse(&input)
}

pub fn render_plan(manifest: &CompositionManifest) -> Result<Plan> {
    manifest.plan()
}

pub fn apply(manifest: CompositionManifest, state_dir: &Path, dry_run: bool) -> Result<ApplyOutput> {
    apply_with_lifecycle_client(manifest, state_dir, dry_run, &ProcessLifecycleClient)
}

pub fn apply_with_lifecycle_client(
    manifest: CompositionManifest,
    state_dir: &Path,
    dry_run: bool,
    lifecycle_client: &dyn LifecycleClient,
) -> Result<ApplyOutput> {
    let plan = manifest.plan()?;
    let lifecycle = plan_lifecycle(&manifest, &plan.manifest_sha256)?;
    let run_id = format!(
        "{}-{}",
        sanitize_name(&manifest.metadata.name),
        &plan.manifest_sha256[..12]
    );
    let provider = runtime_provider_name(&manifest.runtime.provider).to_owned();
    if dry_run {
        return Ok(ApplyOutput {
            run_id,
            manifest_sha256: plan.manifest_sha256,
            provider,
            dry_run: true,
            mutation: false,
            lifecycle,
            containers: BTreeMap::new(),
        });
    }
    ensure_apply_capabilities(&manifest)?;
    ensure_service_lifecycle_capability(&manifest)?;
    let state_path = state_path(state_dir, &run_id);
    if state_path.exists() {
        return Err(CliError::conflict(
            "run_exists",
            format!("run {run_id} already has persisted state"),
        ));
    }

    let composer = PrebuiltImageComposer;
    let backend = ContainerBackend::from_provider(&manifest.runtime.provider)?;
    let publisher = ContainerLocalPublisher::new(backend, &manifest.runtime.wsl_distribution);
    publisher.probe()?;
    for intent in &lifecycle.intents {
        if intent.phase != IntentPhase::Create {
            continue;
        }
        let name = &intent.service;
        let service = &manifest.services[name];
        let port_manifest = Manifest::new(name).with_tag("image", &service.image);
        let artifact = composer.compose(&port_manifest).map_err(map_compose_error)?;
        publisher
            .publish(&artifact, &PublishTarget::new(publisher.target_kind(), &service.image))
            .map_err(map_publish_error)?;
    }

    if backend != ContainerBackend::Podman {
        let containers = execute_direct_lifecycle(
            &manifest,
            &lifecycle,
            &run_id,
            &manifest.runtime.wsl_distribution,
        )?;
        let state = RunState {
            state_version: "phenocompose.run/v0".to_owned(),
            run_id: run_id.clone(),
            manifest_sha256: plan.manifest_sha256.clone(),
            provider: provider.clone(),
            created_unix_seconds: unix_seconds()?,
            lifecycle: RunLifecycle::Running,
            containers: containers.clone(),
            manifest,
        };
        if let Err(error) = save_state(state_dir, &state) {
            rollback_containers_from_map(
                &state.manifest.runtime.provider,
                &state.manifest.runtime.wsl_distribution,
                &containers,
            );
            return Err(error);
        }
        return Ok(ApplyOutput {
            run_id,
            manifest_sha256: plan.manifest_sha256,
            provider,
            dry_run: false,
            mutation: true,
            lifecycle,
            containers,
        });
    }

    let request = build_lifecycle_request(&manifest, &lifecycle, &run_id, &plan.manifest_sha256)?;
    let result = match lifecycle_client.execute(&request) {
        Ok(result) => result,
        Err(failure) => {
            rollback_containers_from_map(
                &manifest.runtime.provider,
                &manifest.runtime.wsl_distribution,
                &failure.result.containers,
            );
            return Err(failure.error);
        }
    };
    if let Err(error) = validate_lifecycle_result(&request, &result) {
        rollback_containers_from_map(
            &manifest.runtime.provider,
            &manifest.runtime.wsl_distribution,
            &result.containers,
        );
        return Err(error);
    }
    let containers = result.containers;

    let state = RunState {
        state_version: "phenocompose.run/v0".to_owned(),
        run_id: run_id.clone(),
        manifest_sha256: plan.manifest_sha256.clone(),
        provider: provider.clone(),
        created_unix_seconds: unix_seconds()?,
        lifecycle: RunLifecycle::Running,
        containers: containers.clone(),
        manifest,
    };
    if let Err(error) = save_state(state_dir, &state) {
        rollback_containers_from_map(
            &state.manifest.runtime.provider,
            &state.manifest.runtime.wsl_distribution,
            &containers,
        );
        return Err(error);
    }
    Ok(ApplyOutput {
        run_id,
        manifest_sha256: plan.manifest_sha256,
        provider,
        dry_run: false,
        mutation: true,
        lifecycle,
        containers,
    })
}

pub fn status(state_dir: &Path, run_id: &str) -> Result<StatusOutput> {
    let state = load_state(state_dir, run_id)?;
    ensure_persisted_provider(&state)?;
    let runtime = ContainerRuntime::bare(
        &state.manifest.runtime.provider,
        &state.manifest.runtime.wsl_distribution,
    )?;
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
    let runtime = ContainerRuntime::bare(
        &state.manifest.runtime.provider,
        &state.manifest.runtime.wsl_distribution,
    )?;
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

pub fn run_action(state_dir: &Path, run_id: &str, action: &str, job_id: &str) -> Result<JobProvenance> {
    run_action_with_client(state_dir, run_id, action, job_id, &ProcessEvaluationClient)
}

pub fn run_action_with_client(
    state_dir: &Path,
    run_id: &str,
    action_name: &str,
    job_id: &str,
    client: &dyn EvaluationClient,
) -> Result<JobProvenance> {
    let workspace_base = env::current_dir().map_err(|error| CliError::io("current_dir", error))?;
    run_action_with_client_at(&workspace_base, state_dir, run_id, action_name, job_id, client)
}

pub fn run_action_with_client_at(
    workspace_base: &Path,
    state_dir: &Path,
    run_id: &str,
    action_name: &str,
    job_id: &str,
    client: &dyn EvaluationClient,
) -> Result<JobProvenance> {
    validate_slug("run_id", run_id)?;
    validate_slug("job_id", job_id)?;
    let state = load_state(state_dir, run_id)?;
    if state.lifecycle != RunLifecycle::Running {
        return Err(CliError::conflict(
            "run_not_running",
            format!("run {run_id} is not running"),
        ));
    }
    ensure_action_route(&state)?;
    let action = state.manifest.actions.get(action_name).ok_or_else(|| {
        CliError::not_found(
            "action_not_found",
            format!("run {run_id} has no action named {action_name}"),
        )
    })?;
    let service = &state.manifest.services[&action.service];
    let (dependency_services, gpu_bindings) = action_gpu_bindings(&state.manifest, &action.service)?;
    let command = action.command.clone();
    let mut job = JobProvenance {
        provenance_version: "phenocompose.job-provenance/v1".to_owned(),
        run_id: run_id.to_owned(),
        job_id: job_id.to_owned(),
        manifest_sha256: state.manifest_sha256.clone(),
        effective_engine: "podman".to_owned(),
        resolved_provider: "podman".to_owned(),
        execution_plane: "nanovms".to_owned(),
        action: action_name.to_owned(),
        service: action.service.clone(),
        command: command.clone(),
        image: service.image.clone(),
        dependency_services,
        gpu_bindings: gpu_bindings.clone(),
        timeout_millis: 300_000,
        max_output_bytes: 1_048_576,
        output_root: String::new(),
        output_root_created: false,
        output_root_available_bytes: None,
        toolkit_version: None,
        toolkit_root: None,
        toolkit_executable: None,
        lifecycle: Some(nvms::Lifecycle::local_failure(b"", b"")),
        success: false,
        error_code: String::new(),
        error_message: String::new(),
    };
    if job_path(state_dir, run_id, job_id).exists() {
        return Err(CliError::conflict(
            "job_exists",
            format!("job {job_id} already has persisted provenance"),
        ));
    }

    let request = match build_evaluation_request(workspace_base, state_dir, &state, &job, service) {
        Ok(request) => request,
        Err(error) => return persist_job_failure(state_dir, job, error),
    };
    job.output_root.clone_from(&request.output_root);
    let result = match client.execute(&request) {
        Ok(result) => result,
        Err(failure) => {
            let nvms::EvaluationFailure { error, lifecycle } = *failure;
            job.lifecycle = Some(trustworthy_failure_lifecycle(lifecycle, request.max_output_bytes));
            return persist_job_failure(state_dir, job, error);
        }
    };
    job.output_root_created = result.provenance.output_root_created;
    job.output_root_available_bytes = result.provenance.output_root_available_bytes;
    job.toolkit_version = result.provenance.toolkit_version.clone();
    job.toolkit_root = result.provenance.toolkit_root.clone();
    job.toolkit_executable = result.provenance.toolkit_executable.clone();
    job.lifecycle = Some(trustworthy_failure_lifecycle(
        result.lifecycle.clone(),
        request.max_output_bytes,
    ));
    job.error_code = result.error_code.clone();
    job.error_message = result.error_message.clone();
    if let Err(error) = validate_evaluation_result(&request, &result) {
        return persist_job_failure(state_dir, job, error);
    }
    if !result.success {
        let code = if result.error_code.is_empty() {
            "nvms_action_failed"
        } else {
            &result.error_code
        };
        let message = if result.error_message.is_empty() {
            "NanoVMS rejected the evaluation action".to_owned()
        } else {
            result.error_message
        };
        return persist_job_failure(state_dir, job, CliError::backend(code, message));
    }
    job.success = true;
    save_job_provenance(state_dir, &job)?;
    Ok(job)
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
        jobs: load_jobs(state_dir, run_id)?,
    })
}

fn action_gpu_bindings(manifest: &CompositionManifest, root_service: &str) -> Result<(Vec<String>, Vec<GpuBinding>)> {
    fn visit(manifest: &CompositionManifest, name: &str, visited: &mut BTreeSet<String>, output: &mut Vec<String>) {
        if !visited.insert(name.to_owned()) {
            return;
        }
        for dependency in &manifest.services[name].depends_on {
            visit(manifest, dependency, visited, output);
        }
        output.push(name.to_owned());
    }

    let mut closure = Vec::new();
    visit(manifest, root_service, &mut BTreeSet::new(), &mut closure);
    let toolkit = &manifest.environment.toolkit;
    let mut uuids = BTreeSet::new();
    for name in &closure {
        let Some(gpu) = manifest.services[name]
            .resources
            .as_ref()
            .and_then(|resources| resources.gpu.as_ref())
        else {
            continue;
        };
        if gpu.vendor != GpuVendor::Nvidia {
            return Err(CliError::unsupported(
                "gpu_vendor_unsupported",
                format!("action dependency {name} requires a non-NVIDIA GPU"),
                "evaluation.gpu_uuid",
                "nanovms",
            ));
        }
        if !toolkit.name.eq_ignore_ascii_case("cuda") || toolkit.version.trim().is_empty() {
            return Err(CliError::validation(
                "toolkit_missing",
                format!("GPU-bearing action dependency {name} requires an explicit CUDA toolkit"),
            ));
        }
        for uuid in &gpu.uuids {
            let canonical = canonical_gpu_uuid(uuid).ok_or_else(|| {
                CliError::validation(
                    "gpu_selector_invalid",
                    format!("action dependency {name} has invalid GPU UUID {uuid:?}"),
                )
            })?;
            uuids.insert(canonical);
        }
    }
    if uuids.is_empty() {
        return Err(CliError::validation(
            "gpu_binding_missing",
            "NanoVMS evaluation actions require at least one GPU in the service dependency closure",
        ));
    }
    let bindings = uuids
        .into_iter()
        .map(|uuid| GpuBinding {
            cdi_device: format!("nvidia.com/gpu={uuid}"),
            uuid,
            cuda_toolkit: toolkit.version.clone(),
        })
        .collect();
    Ok((closure, bindings))
}

fn ensure_action_route(state: &RunState) -> Result<()> {
    let compatibility = state.manifest.runtime.portage_compatibility.as_ref();
    let exact_route = state.provider == "podman"
        && state.manifest.runtime.provider == RuntimeProvider::Podman
        && compatibility.is_some_and(|value| {
            value.external_engine_token == ExternalEngineToken::Docker
                && value.effective_engine == EffectiveEngine::Podman
        });
    if exact_route {
        Ok(())
    } else {
        Err(CliError::unsupported(
            "action_route_rejected",
            "evaluation requires the exact Podman route and historical docker schema token without fallback",
            "evaluation.action",
            &state.provider,
        ))
    }
}

fn build_evaluation_request(
    workspace_base: &Path,
    state_dir: &Path,
    state: &RunState,
    job: &JobProvenance,
    service: &Service,
) -> Result<EvaluationRequest> {
    let (executable, argv) = job
        .command
        .split_first()
        .ok_or_else(|| CliError::validation("action_command_empty", "action command must not be empty"))?;
    let output_root = state
        .manifest
        .actions
        .get(&job.action)
        .and_then(|action| action.output_root.as_deref())
        .ok_or_else(|| {
            CliError::validation(
                "action_output_root_missing",
                format!("action {} must declare output_root", job.action),
            )
        })
        .and_then(|root| resolve_output_root(workspace_base, root))?;
    let state_dir = absolute_path(state_dir)?;
    let reservation_path = state_dir.join("nanovms-gpu-reservations.json");
    let mut environment = state.manifest.environment.variables.clone();
    environment.extend(service.environment.clone());
    let gpus = job
        .gpu_bindings
        .iter()
        .map(|binding| ResourceGpu {
            uuid: binding.uuid.clone(),
            name: None,
            architecture: None,
            compute_capability: None,
            driver_version: String::new(),
            driver_cuda_ceiling: String::new(),
            observations: Vec::new(),
        })
        .collect();
    Ok(EvaluationRequest {
        version: ACTION_VERSION.to_owned(),
        backend: "podman".to_owned(),
        fallback_backends: Vec::new(),
        manifest_sha256: state.manifest_sha256.clone(),
        executable: executable.clone(),
        argv: argv.to_vec(),
        environment,
        external_engine_token: "docker".to_owned(),
        podman_pipe: DEFAULT_PODMAN_PIPE.to_owned(),
        wsl_distribution: state.manifest.runtime.wsl_distribution.clone().unwrap_or_default(),
        output_root: path_string(&output_root)?,
        reservation_path: path_string(&reservation_path)?,
        lock_invocation: job.command.clone(),
        resource_manifest: ResourceManifest {
            version: RESOURCE_VERSION.to_owned(),
            gpus,
            artifact: ArtifactRequirements {
                cuda_toolkit: job.gpu_bindings[0].cuda_toolkit.clone(),
                compiled_kernels: true,
            },
        },
        gpu_bindings: job.gpu_bindings.clone(),
        timeout_millis: job.timeout_millis,
        max_output_bytes: job.max_output_bytes,
    })
}

fn validate_evaluation_result(request: &EvaluationRequest, result: &EvaluationResult) -> Result<()> {
    if result.version != ACTION_VERSION {
        return Err(CliError::backend(
            "nvms_version_mismatch",
            format!("unexpected NanoVMS action version {:?}", result.version),
        ));
    }
    let provenance = &result.provenance;
    if !provenance
        .manifest_sha256
        .eq_ignore_ascii_case(&request.manifest_sha256)
    {
        return Err(CliError::backend(
            "nvms_manifest_mismatch",
            "NanoVMS returned a different manifest digest",
        ));
    }
    if provenance.effective_engine != "podman"
        || provenance.resolved_provider != "podman"
        || provenance.execution_plane != "nanovms"
    {
        return Err(CliError::backend(
            "nvms_route_mismatch",
            "NanoVMS did not attest the exact podman provider through the nanovms execution plane",
        ));
    }
    if provenance.podman_pipe != request.podman_pipe {
        return Err(CliError::backend(
            "nvms_pipe_mismatch",
            "NanoVMS returned a different Podman pipe",
        ));
    }
    if !provenance.job_directory.is_empty() {
        let output_root = Path::new(&request.output_root);
        let job_directory = Path::new(&provenance.job_directory);
        if !job_directory.is_absolute() || job_directory.parent() != Some(output_root) {
            return Err(CliError::backend(
                "nvms_output_root_mismatch",
                "NanoVMS returned a job directory outside the requested output root",
            ));
        }
    }
    let expected: BTreeSet<_> = request.gpu_bindings.iter().map(|binding| &binding.uuid).collect();
    let actual: BTreeSet<_> = provenance.gpu_uuids.iter().collect();
    if expected != actual || actual.len() != provenance.gpu_uuids.len() {
        return Err(CliError::backend(
            "nvms_gpu_binding_mismatch",
            "NanoVMS returned different or duplicate GPU UUID bindings",
        ));
    }
    let lifecycle = &result.lifecycle;
    if lifecycle.duration_ms < 0
        || lifecycle.duration_ms > request.timeout_millis
        || lifecycle.stdout.len() > request.max_output_bytes
        || lifecycle.stderr.len() > request.max_output_bytes
    {
        return Err(CliError::backend(
            "nvms_lifecycle_limits",
            "NanoVMS lifecycle evidence exceeded the requested bounds",
        ));
    }
    validate_evidence_hash("stdout", lifecycle.stdout.as_bytes(), &lifecycle.stdout_sha256)?;
    validate_evidence_hash("stderr", lifecycle.stderr.as_bytes(), &lifecycle.stderr_sha256)?;
    if result.success
        && (!result.released
            || lifecycle.exit_code != 0
            || lifecycle.timed_out
            || lifecycle.truncated
            || !result.error_code.is_empty()
            || !result.error_message.is_empty())
    {
        return Err(CliError::backend(
            "nvms_success_inconsistent",
            "NanoVMS success result contains failure lifecycle evidence",
        ));
    }
    Ok(())
}

fn validate_evidence_hash(label: &str, bytes: &[u8], claimed: &str) -> Result<()> {
    if claimed.len() != 64 || !claimed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::backend(
            "nvms_evidence_hash_invalid",
            format!("NanoVMS {label} hash is not a SHA-256 hex digest"),
        ));
    }
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(claimed) {
        return Err(CliError::backend(
            "nvms_evidence_hash_mismatch",
            format!("NanoVMS {label} hash does not match bounded evidence"),
        ));
    }
    Ok(())
}

fn trustworthy_failure_lifecycle(mut lifecycle: nvms::Lifecycle, max_output_bytes: usize) -> nvms::Lifecycle {
    lifecycle.stdout = bounded_string(lifecycle.stdout, max_output_bytes, &mut lifecycle.truncated);
    lifecycle.stderr = bounded_string(lifecycle.stderr, max_output_bytes, &mut lifecycle.truncated);
    if validate_evidence_hash("stdout", lifecycle.stdout.as_bytes(), &lifecycle.stdout_sha256).is_err() {
        lifecycle.stdout_sha256 = format!("{:x}", Sha256::digest(lifecycle.stdout.as_bytes()));
    }
    if validate_evidence_hash("stderr", lifecycle.stderr.as_bytes(), &lifecycle.stderr_sha256).is_err() {
        lifecycle.stderr_sha256 = format!("{:x}", Sha256::digest(lifecycle.stderr.as_bytes()));
    }
    lifecycle
}

fn bounded_string(mut value: String, limit: usize, truncated: &mut bool) -> String {
    if value.len() <= limit {
        return value;
    }
    let mut boundary = limit;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
    *truncated = true;
    value
}

fn persist_job_failure<T>(state_dir: &Path, mut job: JobProvenance, error: CliError) -> Result<T> {
    job.success = false;
    if job.error_code.is_empty() {
        job.error_code = error.code.clone();
    }
    if job.error_message.is_empty() {
        job.error_message = error.message.clone();
    }
    save_job_provenance(state_dir, &job)?;
    Err(error)
}

fn save_job_provenance(state_dir: &Path, job: &JobProvenance) -> Result<()> {
    let path = job_path(state_dir, &job.run_id, &job.job_id);
    let parent = path
        .parent()
        .ok_or_else(|| CliError::validation("job_path", "job provenance path has no parent"))?;
    fs::create_dir_all(parent).map_err(|error| CliError::io("job_dir_create", error))?;
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let bytes = serde_json::to_vec_pretty(job).map_err(CliError::json)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| CliError::io("job_write", error))?;
    let write_result = file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| CliError::io("job_write", error));
    drop(file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        CliError::io("job_commit", error)
    })
}

pub fn load_job_provenance(state_dir: &Path, run_id: &str, job_id: &str) -> Result<JobProvenance> {
    validate_slug("run_id", run_id)?;
    validate_slug("job_id", job_id)?;
    let input =
        fs::read_to_string(job_path(state_dir, run_id, job_id)).map_err(|error| CliError::io("job_read", error))?;
    serde_json::from_str(&input).map_err(CliError::json)
}

fn load_jobs(state_dir: &Path, run_id: &str) -> Result<BTreeMap<String, JobProvenance>> {
    let directory = jobs_path(state_dir, run_id);
    if !directory.exists() {
        return Ok(BTreeMap::new());
    }
    let mut jobs = BTreeMap::new();
    for entry in fs::read_dir(directory).map_err(|error| CliError::io("jobs_read", error))? {
        let entry = entry.map_err(|error| CliError::io("jobs_read", error))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let input = fs::read_to_string(&path).map_err(|error| CliError::io("job_read", error))?;
        let job: JobProvenance = serde_json::from_str(&input).map_err(CliError::json)?;
        if job.run_id != run_id || path.file_stem().and_then(|value| value.to_str()) != Some(&job.job_id) {
            return Err(CliError::validation(
                "job_identity_mismatch",
                format!("job provenance identity does not match {}", path.display()),
            ));
        }
        jobs.insert(job.job_id.clone(), job);
    }
    Ok(jobs)
}

fn validate_slug(field: &str, value: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.starts_with(|character: char| character.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(CliError::validation(
            format!("{field}_invalid"),
            format!("{field} must be a path-safe ASCII slug"),
        ))
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| CliError::io("current_dir", error))
    }
}

fn resolve_output_root(workspace_base: &Path, declared: &str) -> Result<PathBuf> {
    let root = Path::new(declared);
    if declared.trim().is_empty() {
        return Err(CliError::validation(
            "action_output_root_empty",
            "action output_root must not be empty",
        ));
    }
    if root.has_root() && !root.is_absolute() {
        return Err(CliError::validation(
            "action_output_root_ambiguous",
            "action output_root must be fully absolute or relative to the workspace",
        ));
    }
    if root
        .components()
        .any(|component| matches!(component, Component::Prefix(_)))
        && !root.is_absolute()
    {
        return Err(CliError::validation(
            "action_output_root_ambiguous",
            "action output_root must not use a drive-relative path",
        ));
    }
    let base = absolute_path(workspace_base)?;
    let resolved = if root.is_absolute() {
        root.to_owned()
    } else {
        base.join(root)
    };
    normalize_output_root(&resolved)
}

fn normalize_output_root(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(CliError::validation(
                    "action_output_root_traversal",
                    "action output_root must not contain parent traversal",
                ));
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    if !normalized.is_absolute() {
        return Err(CliError::validation(
            "action_output_root_ambiguous",
            "resolved action output_root must be absolute",
        ));
    }
    Ok(normalized)
}

fn path_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| CliError::validation("path_encoding", "NanoVMS paths must be valid UTF-8"))
}

fn ensure_apply_capabilities(manifest: &CompositionManifest) -> Result<()> {
    match manifest.runtime.provider {
        RuntimeProvider::Podman | RuntimeProvider::AppleContainers | RuntimeProvider::WslContainers => {}
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
        if provider.capability == SERVICE_LIFECYCLE_CAPABILITY {
            continue;
        }
        if provider.status != ProviderStatus::Available {
            return Err(CliError::unsupported(
                "provider_unavailable",
                format!("provider declaration {name} is not available"),
                &provider.capability,
                provider.implementation.as_deref().unwrap_or("placeholder"),
            ));
        }
    }
    let lifecycle_available = manifest.providers.values().any(|provider| {
        provider.capability == SERVICE_LIFECYCLE_CAPABILITY && provider.status == ProviderStatus::Available
    });
    if manifest.runtime.provider == RuntimeProvider::Podman
        && lifecycle_available
        && manifest.services.values().any(|service| service.health_check.is_some())
    {
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
        if manifest.runtime.provider != RuntimeProvider::Podman
            && service.resources.as_ref().is_some_and(|resources| {
                resources.cpu_millis.is_some()
                    || resources.memory_bytes.is_some()
                    || resources.gpu.is_some()
            })
        {
            return Err(CliError::unsupported(
                "runtime_resource_flags_unsupported",
                format!(
                    "{} does not expose portable CPU, memory, or GPU resource flags",
                    runtime_provider_name(&manifest.runtime.provider)
                ),
                "runtime.resources",
                runtime_provider_name(&manifest.runtime.provider),
            ));
        }
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
        RuntimeProvider::AppleContainers => "apple-containers",
        RuntimeProvider::WslContainers => "wsl-containers",
        RuntimeProvider::Nvms => "nvms",
        RuntimeProvider::Placeholder => "placeholder",
    }
}

fn ensure_persisted_provider(state: &RunState) -> Result<()> {
    if matches!(state.provider.as_str(), "podman" | "apple-containers" | "wsl-containers") {
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

fn rollback_containers_from_map(
    provider: &RuntimeProvider,
    distribution: &Option<String>,
    containers: &BTreeMap<String, String>,
) {
    let Ok(runtime) = ContainerRuntime::bare(provider, distribution) else {
        return;
    };
    for id in containers.values().rev() {
        let _ = runtime.stop(&ContainerId::new(id));
    }
}

fn execute_direct_lifecycle(
    manifest: &CompositionManifest,
    lifecycle: &LifecyclePlan,
    run_id: &str,
    distribution: &Option<String>,
) -> Result<BTreeMap<String, String>> {
    let mut rollback = RollbackContract::default();
    let runtime = ContainerRuntime::bare(&manifest.runtime.provider, distribution)?;
    let mut containers = BTreeMap::new();

    for service_name in &lifecycle.order {
        let service = &manifest.services[service_name];
        let service_runtime = ContainerRuntime::for_service(
            &manifest.runtime.provider,
            distribution,
            run_id,
            service_name,
            service,
        )?;
        match service_runtime.spawn(&ImageRef::new(&service.image)) {
            Ok(id) => {
                rollback.record_create(service_name, id.clone());
                containers.insert(service_name.clone(), id.id);
            }
            Err(error) => {
                rollback.rollback(&runtime);
                return Err(map_runtime_error(error));
            }
        }
    }

    for intent in lifecycle.intents.iter().filter(|intent| intent.phase == IntentPhase::Start) {
        let Some(check) = intent.health_check.as_ref() else {
            continue;
        };
        let service = &manifest.services[&intent.service];
        let service_runtime = ContainerRuntime::for_service(
            &manifest.runtime.provider,
            distribution,
            run_id,
            &intent.service,
            service,
        )?;
        let id = ContainerId::new(containers[&intent.service].clone());
        if let Err(error) = run_health_check(&service_runtime, &id, check) {
            rollback.rollback(&runtime);
            return Err(error);
        }
    }

    Ok(containers)
}

fn run_health_check(
    runtime: &ContainerRuntime,
    id: &ContainerId,
    check: &crate::service_graph::HealthCheckIntent,
) -> Result<()> {
    let attempts = check.retries.saturating_add(1);
    let timeout = Duration::from_secs(u64::from(check.timeout_seconds));
    let mut last_error = None;
    for attempt in 0..attempts {
        match runtime.health_with_timeout(id, &check.command, timeout) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
        if attempt + 1 < attempts && check.interval_seconds > 0 {
            thread::sleep(Duration::from_secs(u64::from(check.interval_seconds)));
        }
    }
    Err(CliError::backend(
        "runtime_health_failed",
        format!(
            "{} health check failed after {} attempt(s): {}",
            runtime.name(),
            attempts,
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "unknown runtime error".to_owned())
        ),
    ))
}

fn build_lifecycle_request(
    manifest: &CompositionManifest,
    plan: &LifecyclePlan,
    run_id: &str,
    manifest_sha256: &str,
) -> Result<ServiceLifecycleRequest> {
    let mut services = BTreeMap::new();
    for (name, service) in &manifest.services {
        let resources = service.resources.as_ref();
        services.insert(
            name.clone(),
            ServiceDefinition {
                image: service.image.clone(),
                depends_on: service.depends_on.clone(),
                command: service.command.clone(),
                environment: service.environment.clone(),
                cpu_millis: resources.and_then(|value| value.cpu_millis),
                memory_bytes: resources.and_then(|value| value.memory_bytes),
                gpu_uuids: resources
                    .and_then(|value| value.gpu.as_ref())
                    .map(|gpu| gpu.uuids.clone())
                    .unwrap_or_default(),
            },
        );
    }
    let intents = plan
        .intents
        .iter()
        .map(|intent| ServiceLifecycleIntent {
            phase: match intent.phase {
                IntentPhase::Create => "create".to_owned(),
                IntentPhase::Start => "start".to_owned(),
            },
            service: intent.service.clone(),
            image: intent.image.clone(),
            depends_on: intent.depends_on.clone(),
        })
        .collect();
    Ok(ServiceLifecycleRequest {
        version: SERVICE_LIFECYCLE_VERSION.to_owned(),
        schema_version: LIFECYCLE_SCHEMA_VERSION.to_owned(),
        manifest_sha256: manifest_sha256.to_owned(),
        run_id: run_id.to_owned(),
        wsl_distribution: manifest.runtime.wsl_distribution.clone().unwrap_or_default(),
        podman_pipe: DEFAULT_PODMAN_PIPE.to_owned(),
        order: plan.order.clone(),
        intents,
        services,
    })
}

fn validate_lifecycle_result(request: &ServiceLifecycleRequest, result: &ServiceLifecycleResult) -> Result<()> {
    if result.version != SERVICE_LIFECYCLE_VERSION {
        return Err(CliError::backend(
            "nvms_version_mismatch",
            format!("unexpected NanoVMS lifecycle version {:?}", result.version),
        ));
    }
    if !result.success {
        let code = if result.error_code.is_empty() {
            "nvms_lifecycle_failed".to_owned()
        } else {
            result.error_code.clone()
        };
        return Err(CliError::backend(code, result.error_message.clone()));
    }
    if result.effective_engine != "podman" || result.resolved_provider != "podman" {
        return Err(CliError::backend(
            "nvms_route_mismatch",
            "NanoVMS did not attest the podman provider for service lifecycle",
        ));
    }
    if result.podman_pipe != request.podman_pipe {
        return Err(CliError::backend(
            "nvms_pipe_mismatch",
            "NanoVMS returned a different Podman pipe",
        ));
    }
    for service in &request.order {
        if !result.containers.contains_key(service) {
            return Err(CliError::backend(
                "nvms_container_missing",
                format!("NanoVMS lifecycle result is missing container for service {service}"),
            ));
        }
    }
    Ok(())
}

fn state_path(state_dir: &Path, run_id: &str) -> PathBuf {
    state_dir.join(format!("{run_id}.json"))
}

fn jobs_path(state_dir: &Path, run_id: &str) -> PathBuf {
    state_dir.join(format!("{run_id}.jobs"))
}

fn job_path(state_dir: &Path, run_id: &str, job_id: &str) -> PathBuf {
    jobs_path(state_dir, run_id).join(format!("{job_id}.json"))
}

fn load_state(state_dir: &Path, run_id: &str) -> Result<RunState> {
    validate_slug("run_id", run_id)?;
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
struct ContainerLocalPublisher {
    backend: ContainerBackend,
    distribution: Option<String>,
}

impl ContainerLocalPublisher {
    fn new(backend: ContainerBackend, distribution: &Option<String>) -> Self {
        Self {
            backend,
            distribution: distribution.clone(),
        }
    }

    fn probe(&self) -> Result<()> {
        let arguments = if self.backend == ContainerBackend::Podman {
            vec!["version".to_owned(), "--format".to_owned(), "json".to_owned()]
        } else {
            vec!["--help".to_owned()]
        };
        let output = runtime_output_owned(self.backend, &self.distribution, &arguments)?;
        let code = if self.backend == ContainerBackend::Podman {
            "podman_unavailable"
        } else {
            "runtime_unavailable"
        };
        require_success(code, output).map(|_| ())
    }

    fn target_kind(&self) -> &'static str {
        self.backend.target_kind()
    }
}

impl Publisher for ContainerLocalPublisher {
    fn publish(
        &self,
        artifact: &ComposedArtifact,
        target: &PublishTarget,
    ) -> std::result::Result<PublishReceipt, PublishError> {
        if target.kind != self.target_kind() || target.locator != artifact.image.as_ref() {
            return Err(PublishError::validation(
                format!("{} target must match the composed image", self.target_kind()),
            ));
        }
        let arguments = self.backend.image_inspect_arguments(&artifact.image);
        let output = runtime_output_owned(self.backend, &self.distribution, &arguments)
            .map_err(|error| PublishError::transport(error.to_string()))?;
        if !output.status.success() {
            return Err(PublishError::transport(format!(
                "image {} is not present in {} local storage",
                artifact.image,
                self.backend.display_name(),
            )));
        }
        Ok(PublishReceipt::new(
            &artifact.id,
            target.clone(),
            format!("{}://{}", self.target_kind(), artifact.image),
        ))
    }

    fn name(&self) -> &str {
        self.target_kind()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerBackend {
    Podman,
    AppleContainers,
    WslContainers,
}

impl ContainerBackend {
    fn from_provider(provider: &RuntimeProvider) -> Result<Self> {
        match provider {
            RuntimeProvider::Podman => Ok(Self::Podman),
            RuntimeProvider::AppleContainers => Ok(Self::AppleContainers),
            RuntimeProvider::WslContainers => Ok(Self::WslContainers),
            RuntimeProvider::Nvms | RuntimeProvider::Placeholder => Err(CliError::unsupported(
                "runtime_provider_unsupported",
                format!("{} is not a local container backend", runtime_provider_name(provider)),
                "runtime.provider",
                runtime_provider_name(provider),
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Podman => "podman",
            Self::AppleContainers => "apple-containers",
            Self::WslContainers => "wsl-containers",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Podman => "Podman",
            Self::AppleContainers => "Apple Containers",
            Self::WslContainers => "WSLc",
        }
    }

    fn command_candidates(self, distribution: &Option<String>) -> Result<Vec<Command>> {
        if self != Self::Podman && distribution.is_some() {
            return Err(CliError::unsupported(
                "runtime_distribution_unsupported",
                format!("{} does not accept a WSL distribution override", self.display_name()),
                "runtime.wsl_distribution",
                self.name(),
            ));
        }
        let commands = match self {
            Self::Podman if distribution.is_some() => {
                let mut command = Command::new("wsl.exe");
                command.args(["-d", distribution.as_deref().unwrap(), "--", "podman"]);
                vec![command]
            }
            Self::Podman => vec![Command::new("podman")],
            Self::AppleContainers => vec![Command::new("container")],
            Self::WslContainers => vec![
                Command::new("wslc"),
                Command::new("wslc.exe"),
                Command::new("container.exe"),
            ],
        };
        Ok(commands)
    }

    fn supports_resource_flags(self) -> bool {
        matches!(self, Self::Podman)
    }

    fn target_kind(self) -> &'static str {
        match self {
            Self::Podman => "podman-local",
            Self::AppleContainers => "apple-containers-local",
            Self::WslContainers => "wsl-containers-local",
        }
    }

    fn image_inspect_arguments(self, image: &ImageRef) -> Vec<String> {
        let operation = if self == Self::Podman { "exists" } else { "inspect" };
        vec!["image".to_owned(), operation.to_owned(), image.to_string()]
    }

    fn stop_arguments(self, id: &ContainerId) -> Vec<String> {
        match self {
            Self::Podman | Self::AppleContainers => vec!["stop".to_owned(), id.to_string()],
            Self::WslContainers => vec!["container".to_owned(), "stop".to_owned(), id.to_string()],
        }
    }

    fn inspect_arguments(self, id: &ContainerId) -> Vec<String> {
        match self {
            Self::Podman => vec![
                "inspect".to_owned(),
                "--format".to_owned(),
                "{{.State.Status}}".to_owned(),
                id.to_string(),
            ],
            Self::AppleContainers => vec!["inspect".to_owned(), id.to_string()],
            Self::WslContainers => vec!["container".to_owned(), "inspect".to_owned(), id.to_string()],
        }
    }

    fn probe_arguments(self) -> Vec<Vec<String>> {
        match self {
            Self::Podman => vec![vec!["info".to_owned(), "--format".to_owned(), "json".to_owned()]],
            Self::AppleContainers => vec![
                vec![
                    "system".to_owned(),
                    "status".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned(),
                ],
                vec![
                    "system".to_owned(),
                    "version".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned(),
                ],
            ],
            Self::WslContainers => vec![vec!["image".to_owned(), "ls".to_owned()]],
        }
    }

    fn probe_commands(self, distribution: &Option<String>) -> Result<Vec<Vec<String>>> {
        if self != Self::Podman && distribution.is_some() {
            return Err(CliError::unsupported(
                "runtime_distribution_unsupported",
                format!("{} does not accept a WSL distribution override", self.display_name()),
                "runtime.wsl_distribution",
                self.name(),
            ));
        }

        let prefix = match self {
            Self::Podman if distribution.is_some() => vec![
                "wsl.exe".to_owned(),
                "-d".to_owned(),
                distribution.as_deref().unwrap_or_default().to_owned(),
                "--".to_owned(),
                "podman".to_owned(),
            ],
            Self::Podman => vec!["podman".to_owned()],
            Self::AppleContainers => vec!["container".to_owned()],
            Self::WslContainers => vec!["wslc.exe".to_owned()],
        };

        Ok(self
            .probe_arguments()
            .into_iter()
            .map(|arguments| {
                let mut command = prefix.clone();
                command.extend(arguments);
                command
            })
            .collect())
    }
}

/// Probe a local container backend without starting or stopping any runtime
/// resource.
pub fn probe_runtime(provider: &RuntimeProvider, distribution: &Option<String>) -> Result<CapabilityReceipt> {
    let backend = ContainerBackend::from_provider(provider)?;
    let expected_commands = backend.probe_commands(distribution)?;
    let mut commands = Vec::with_capacity(expected_commands.len());
    let mut outputs = Vec::with_capacity(expected_commands.len());

    for arguments in backend.probe_arguments() {
        let (command, output) =
            runtime_output_owned_with_command(backend, distribution, &arguments, PODMAN_COMMAND_TIMEOUT)
                .map_err(|error| CliError::backend("runtime_probe_failed", error.to_string()))?;
        commands.push(command);
        if !output.status.success() {
            return Err(CliError::backend(
                "runtime_probe_failed",
                format!("{} probe failed: {}", backend.display_name(), output_message(&output)),
            ));
        }
        if matches!(backend, ContainerBackend::Podman | ContainerBackend::AppleContainers) {
            serde_json::from_slice::<serde_json::Value>(&output.stdout).map_err(|error| {
                CliError::backend(
                    "runtime_probe_invalid_output",
                    format!("{} probe returned invalid JSON: {error}", backend.display_name()),
                )
            })?;
        }
        outputs.push(output);
    }

    let mut digest = Sha256::new();
    let mut version = None;
    for output in &outputs {
        digest.update(&output.stdout);
        digest.update([0]);
        digest.update(&output.stderr);
        digest.update([0]);
        if version.is_none() {
            version = extract_runtime_version(&output.stdout);
        }
    }

    Ok(CapabilityReceipt {
        schema_version: "phenocompose.runtime-capability/v1".to_owned(),
        provider: backend.name().to_owned(),
        executable: commands
            .first()
            .and_then(|command| command.first())
            .cloned()
            .unwrap_or_else(|| backend.name().to_owned()),
        commands,
        ready: true,
        version,
        output_sha256: format!("{:x}", digest.finalize()),
    })
}

/// Parse the stable provider spellings accepted by the probe command.
pub fn parse_runtime_provider(value: &str) -> Result<RuntimeProvider> {
    match value.trim().to_ascii_lowercase().as_str() {
        "podman" => Ok(RuntimeProvider::Podman),
        "apple" | "apple-containers" | "apple_containers" | "container" => Ok(RuntimeProvider::AppleContainers),
        "wslc" | "wslc.exe" | "wsl-containers" | "wsl_containers" => Ok(RuntimeProvider::WslContainers),
        "nvms" => Ok(RuntimeProvider::Nvms),
        "placeholder" => Ok(RuntimeProvider::Placeholder),
        other => Err(CliError::validation(
            "runtime_provider_invalid",
            format!("unsupported runtime provider {other:?}"),
        )),
    }
}

fn extract_runtime_version(output: &[u8]) -> Option<String> {
    let value = serde_json::from_slice::<serde_json::Value>(output).ok()?;
    find_runtime_version(&value)
}

fn find_runtime_version(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            for key in ["version", "Version", "clientVersion", "ClientVersion"] {
                if let Some(candidate) = map.get(key) {
                    if let Some(version) = candidate
                        .as_str()
                        .map(str::trim)
                        .filter(|version| !version.is_empty())
                    {
                        return Some(version.to_owned());
                    }
                    if let Some(version) = find_runtime_version(candidate) {
                        return Some(version);
                    }
                }
            }
            for candidate in map.values() {
                if let Some(version) = find_runtime_version(candidate) {
                    return Some(version);
                }
            }
            None
        }
        serde_json::Value::Array(values) => values.iter().find_map(find_runtime_version),
        _ => None,
    }
}

#[derive(Debug)]
struct ContainerRuntime {
    backend: ContainerBackend,
    distribution: Option<String>,
    container_name: Option<String>,
    command: Vec<String>,
    environment: BTreeMap<String, String>,
    cpu_millis: Option<u32>,
    memory_bytes: Option<u64>,
    gpu_uuids: Vec<String>,
}

impl ContainerRuntime {
    fn bare(provider: &RuntimeProvider, distribution: &Option<String>) -> Result<Self> {
        let backend = ContainerBackend::from_provider(provider)?;
        Ok(Self {
            backend,
            distribution: distribution.clone(),
            container_name: None,
            command: Vec::new(),
            environment: BTreeMap::new(),
            cpu_millis: None,
            memory_bytes: None,
            gpu_uuids: Vec::new(),
        })
    }

    fn for_service(
        provider: &RuntimeProvider,
        distribution: &Option<String>,
        run_id: &str,
        service_name: &str,
        service: &Service,
    ) -> Result<Self> {
        let mut runtime = Self::bare(provider, distribution)?;
        let resources = service.resources.as_ref();
        runtime.container_name = Some(format!("{run_id}-{service_name}"));
        runtime.command = service.command.clone();
        runtime.environment = service.environment.clone();
        runtime.cpu_millis = resources.and_then(|value| value.cpu_millis);
        runtime.memory_bytes = resources.and_then(|value| value.memory_bytes);
        runtime.gpu_uuids = resources
            .and_then(|value| value.gpu.as_ref())
            .map(|gpu| gpu.uuids.clone())
            .unwrap_or_default();
        Ok(runtime)
    }

    fn spawn(&self, image: &ImageRef) -> std::result::Result<ContainerId, RuntimeError> {
        let name = self
            .container_name
            .as_ref()
            .ok_or_else(|| RuntimeError::validation("service configuration is required"))?;
        if !self.backend.supports_resource_flags()
            && (self.cpu_millis.is_some() || self.memory_bytes.is_some() || !self.gpu_uuids.is_empty())
        {
            return Err(RuntimeError::validation(format!(
                "{} does not support CPU, memory, or GPU resource flags",
                self.backend.display_name()
            )));
        }
        let mut arguments = vec![
            "run".to_owned(),
            if self.backend == ContainerBackend::Podman {
                "--detach".to_owned()
            } else {
                "-d".to_owned()
            },
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
        let output = runtime_output_owned(self.backend, &self.distribution, &arguments)
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
        let arguments = self.backend.stop_arguments(id);
        let output = runtime_output_owned(self.backend, &self.distribution, &arguments)
            .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(RuntimeError::backend(output_message(&output)))
        }
    }

    fn status(&self, id: &ContainerId) -> std::result::Result<ContainerStatus, RuntimeError> {
        let arguments = self.backend.inspect_arguments(id);
        let output = runtime_output_owned(self.backend, &self.distribution, &arguments)
        .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if !output.status.success() {
            let message = output_message(&output);
            if message.to_ascii_lowercase().contains("no such") {
                return Ok(ContainerStatus::NotFound);
            }
            return Err(RuntimeError::backend(message));
        }
        let status = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        if status.contains("paused") {
            Ok(ContainerStatus::Paused)
        } else if status.contains("running") {
            Ok(ContainerStatus::Running)
        } else if status.contains("exited")
            || status.contains("stopped")
            || status.contains("configured")
            || status.contains("created")
            || status.contains("dead")
        {
            Ok(ContainerStatus::Exited)
        } else {
            Err(RuntimeError::backend(format!(
                "unrecognized {} status {:?}",
                self.backend.display_name(),
                status.trim()
            )))
        }
    }

    #[cfg(all(unix, test))]
    fn health(&self, id: &ContainerId, command: &[String]) -> std::result::Result<(), RuntimeError> {
        self.health_with_timeout(id, command, PODMAN_COMMAND_TIMEOUT)
    }

    fn health_with_timeout(
        &self,
        id: &ContainerId,
        command: &[String],
        timeout: Duration,
    ) -> std::result::Result<(), RuntimeError> {
        if command.is_empty() {
            return Err(RuntimeError::validation("health command must not be empty"));
        }
        let mut arguments = vec!["exec".to_owned(), id.to_string()];
        arguments.extend(command.iter().cloned());
        let output = runtime_output_owned_with_timeout(self.backend, &self.distribution, &arguments, timeout)
            .map_err(|error| RuntimeError::backend(error.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(RuntimeError::backend(output_message(&output)))
        }
    }

    fn name(&self) -> &str {
        self.backend.name()
    }
}

impl Runtime for ContainerRuntime {
    fn spawn(&self, image: &ImageRef) -> std::result::Result<ContainerId, RuntimeError> {
        ContainerRuntime::spawn(self, image)
    }

    fn stop(&self, id: &ContainerId) -> std::result::Result<(), RuntimeError> {
        ContainerRuntime::stop(self, id)
    }

    fn status(&self, id: &ContainerId) -> std::result::Result<ContainerStatus, RuntimeError> {
        ContainerRuntime::status(self, id)
    }

    fn name(&self) -> &str {
        ContainerRuntime::name(self)
    }
}

fn runtime_output_owned(
    backend: ContainerBackend,
    distribution: &Option<String>,
    arguments: &[String],
) -> Result<Output> {
    runtime_output_owned_with_timeout(backend, distribution, arguments, PODMAN_COMMAND_TIMEOUT)
}

const PODMAN_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(all(unix, test))]
fn podman_output_owned(distribution: &Option<String>, arguments: &[String]) -> Result<Output> {
    podman_output_owned_with_timeout(distribution, arguments, PODMAN_COMMAND_TIMEOUT)
}

#[cfg(all(unix, test))]
fn podman_output_owned_with_timeout(
    distribution: &Option<String>,
    arguments: &[String],
    timeout: Duration,
) -> Result<Output> {
    runtime_output_owned_with_timeout(ContainerBackend::Podman, distribution, arguments, timeout)
}

fn runtime_output_owned_with_timeout(
    backend: ContainerBackend,
    distribution: &Option<String>,
    arguments: &[String],
    timeout: Duration,
) -> Result<Output> {
    runtime_output_owned_with_command(backend, distribution, arguments, timeout).map(|(_, output)| output)
}

fn runtime_output_owned_with_command(
    backend: ContainerBackend,
    distribution: &Option<String>,
    arguments: &[String],
    timeout: Duration,
) -> Result<(Vec<String>, Output)> {
    let unavailable_code = if backend == ContainerBackend::Podman {
        "podman_unavailable"
    } else {
        "runtime_unavailable"
    };
    let mut child = None;
    let mut invoked_command = None;
    let mut last_not_found = None;
    for mut command in backend.command_candidates(distribution)? {
        command.args(arguments);
        let argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        match command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(process) => {
                child = Some(process);
                invoked_command = Some(argv);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                last_not_found = Some(error);
            }
            Err(error) => {
                return Err(CliError::backend(
                    unavailable_code,
                    format!("failed to start {}: {error}", backend.display_name()),
                ));
            }
        }
    }
    let mut child = child.ok_or_else(|| {
        let detail = last_not_found
            .map(|error| error.to_string())
            .unwrap_or_else(|| "no executable candidates were configured".to_owned());
        CliError::backend(
            unavailable_code,
            format!("failed to start {}: {detail}", backend.display_name()),
        )
    })?;

    let invoked_command = invoked_command.expect("a spawned child always records its command");
    match child.wait_timeout(timeout) {
        Ok(Some(_)) => child.wait_with_output().map(|output| (invoked_command, output)).map_err(|error| {
            CliError::backend(
                if backend == ContainerBackend::Podman {
                    "podman_unavailable"
                } else {
                    "runtime_unavailable"
                },
                format!("{} output was unavailable: {error}", backend.display_name()),
            )
        }),
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(CliError::backend(
                if backend == ContainerBackend::Podman {
                    "podman_timeout"
                } else {
                    "runtime_timeout"
                },
                format!(
                    "{} command timed out after {} ms",
                    backend.display_name(),
                    timeout.as_millis()
                ),
            ))
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(CliError::io(
                if backend == ContainerBackend::Podman {
                    "podman_wait"
                } else {
                    "runtime_wait"
                },
                error,
            ))
        }
    }
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

pub fn map_runtime_error(error: RuntimeError) -> CliError {
    match error {
        RuntimeError::NotFound(message) => CliError::not_found("runtime_not_found", message),
        RuntimeError::Validation(message) => CliError::validation("runtime_validation", message),
        RuntimeError::Backend(message) => CliError::backend("runtime_backend", message),
        other => CliError::backend("runtime_unknown", other.to_string()),
    }
}

#[cfg(test)]
mod lifecycle_bridge_tests {
    use super::*;
    use crate::model::CompositionManifest;

    #[test]
    fn lifecycle_request_maps_manifest_services_and_intents() {
        let manifest = CompositionManifest::parse(include_str!("../../../examples/composition-v0.yaml")).unwrap();
        let plan = manifest.plan().unwrap();
        let lifecycle = plan_lifecycle(&manifest, &plan.manifest_sha256).unwrap();
        let request = build_lifecycle_request(&manifest, &lifecycle, "run-id", &plan.manifest_sha256).unwrap();
        assert_eq!(request.version, SERVICE_LIFECYCLE_VERSION);
        assert_eq!(request.schema_version, LIFECYCLE_SCHEMA_VERSION);
        assert_eq!(request.intents[0].phase, "create");
        assert_eq!(request.services["worker"].gpu_uuids[0], "GPU-123e4567-e89b-12d3-a456-426614174000");
    }

    #[test]
    fn lifecycle_result_validation_requires_podman_attestation() {
        let request = ServiceLifecycleRequest {
            version: SERVICE_LIFECYCLE_VERSION.to_owned(),
            schema_version: LIFECYCLE_SCHEMA_VERSION.to_owned(),
            manifest_sha256: "0".repeat(64),
            run_id: "run".to_owned(),
            wsl_distribution: String::new(),
            podman_pipe: DEFAULT_PODMAN_PIPE.to_owned(),
            order: vec!["worker".to_owned()],
            intents: Vec::new(),
            services: BTreeMap::new(),
        };
        let result = ServiceLifecycleResult {
            version: SERVICE_LIFECYCLE_VERSION.to_owned(),
            success: true,
            error_code: String::new(),
            error_message: String::new(),
            containers: BTreeMap::from([("worker".to_owned(), "abc".to_owned())]),
            effective_engine: "gvisor".to_owned(),
            resolved_provider: "podman".to_owned(),
            podman_pipe: DEFAULT_PODMAN_PIPE.to_owned(),
        };
        let error = validate_lifecycle_result(&request, &result).unwrap_err();
        assert_eq!(error.code, "nvms_route_mismatch");
    }
}

#[cfg(test)]
mod container_backend_command_tests {
    use super::*;

    fn program_names(commands: Vec<Command>) -> Vec<String> {
        commands
            .iter()
            .map(|command| command.get_program().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn wslc_candidates_are_ordered_for_extension_aliases() {
        let names = program_names(ContainerBackend::WslContainers.command_candidates(&None).unwrap());
        assert_eq!(names, ["wslc", "wslc.exe", "container.exe"]);
    }

    #[test]
    fn podman_distribution_route_remains_single_wsl_wrapper() {
        let commands = ContainerBackend::Podman
            .command_candidates(&Some("podman-machine-default".to_owned()))
            .unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].get_program().to_string_lossy(), "wsl.exe");
    }

    #[test]
    fn capability_probe_commands_are_provider_specific_and_read_only() {
        assert_eq!(
            ContainerBackend::Podman.probe_arguments(),
            vec![vec!["info".to_owned(), "--format".to_owned(), "json".to_owned()]]
        );
        assert_eq!(
            ContainerBackend::AppleContainers.probe_arguments(),
            vec![
                vec![
                    "system".to_owned(),
                    "status".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned(),
                ],
                vec![
                    "system".to_owned(),
                    "version".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned(),
                ],
            ]
        );
        assert_eq!(
            ContainerBackend::WslContainers.probe_arguments(),
            vec![vec!["image".to_owned(), "ls".to_owned()]]
        );
        for arguments in ContainerBackend::AppleContainers
            .probe_arguments()
            .into_iter()
            .chain(ContainerBackend::WslContainers.probe_arguments())
        {
            assert!(!arguments
                .iter()
                .any(|argument| { matches!(argument.as_str(), "run" | "start" | "stop" | "rm" | "delete") }));
        }
    }

    #[test]
    fn capability_probe_command_includes_wsl_distribution_wrapper() {
        let commands = ContainerBackend::Podman
            .probe_commands(&Some("FedoraLinux-44".to_owned()))
            .unwrap();
        assert_eq!(
            commands,
            vec![vec![
                "wsl.exe".to_owned(),
                "-d".to_owned(),
                "FedoraLinux-44".to_owned(),
                "--".to_owned(),
                "podman".to_owned(),
                "info".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
            ]]
        );
    }

    #[test]
    fn runtime_version_extraction_is_provider_output_agnostic() {
        assert_eq!(
            extract_runtime_version(br#"{"version":{"Version":"5.8.4"}}"#),
            Some("5.8.4".to_owned())
        );
        assert_eq!(
            extract_runtime_version(br#"{"clientVersion":"1.0.0"}"#),
            Some("1.0.0".to_owned())
        );
        assert_eq!(extract_runtime_version(b"not-json"), None);
    }

    #[test]
    fn provider_aliases_are_stable() {
        assert_eq!(parse_runtime_provider("podman").unwrap(), RuntimeProvider::Podman);
        assert_eq!(
            parse_runtime_provider("apple-containers").unwrap(),
            RuntimeProvider::AppleContainers
        );
        assert_eq!(
            parse_runtime_provider("wslc.exe").unwrap(),
            RuntimeProvider::WslContainers
        );
        assert_eq!(
            parse_runtime_provider("unknown").unwrap_err().code,
            "runtime_provider_invalid"
        );
    }
}

#[cfg(all(unix, test))]
mod podman_command_tests {
    use super::*;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    struct FakeCommandEnvironment {
        path: Option<OsString>,
        log: Option<OsString>,
    }

    impl Drop for FakeCommandEnvironment {
        fn drop(&mut self) {
            restore_environment("PATH", self.path.take());
            restore_environment("PHENOCOMPOSE_PODMAN_TEST_LOG", self.log.take());
        }
    }

    fn restore_environment(name: &str, value: Option<OsString>) {
        if let Some(value) = value {
            env::set_var(name, value);
        } else {
            env::remove_var(name);
        }
    }

    fn environment_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn install_fake_command_path(directory: &Path, log: &Path) -> FakeCommandEnvironment {
        let previous_path = env::var_os("PATH");
        let previous_log = env::var_os("PHENOCOMPOSE_PODMAN_TEST_LOG");
        let mut paths = vec![directory.to_path_buf()];
        if let Some(existing) = previous_path.as_ref() {
            paths.extend(env::split_paths(existing));
        }
        env::set_var("PATH", env::join_paths(paths).unwrap());
        env::set_var("PHENOCOMPOSE_PODMAN_TEST_LOG", log);
        FakeCommandEnvironment {
            path: previous_path,
            log: previous_log,
        }
    }

    fn write_fake_command(directory: &Path, name: &str, script: &str) {
        let path = directory.join(name);
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn direct_service() -> Service {
        Service {
            image: "example:latest".to_owned(),
            depends_on: Vec::new(),
            command: vec!["/bin/worker".to_owned()],
            environment: BTreeMap::from([("MODE".to_owned(), "test".to_owned())]),
            resources: None,
            health_check: None,
        }
    }

    fn direct_runtime_script() -> &'static str {
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PHENOCOMPOSE_PODMAN_TEST_LOG\"\ncase \"$1\" in\n  run) printf 'fake-id\\n' ;;\n  image) printf '{}\\n' ;;\n  inspect) printf 'running\\n' ;;\n  exec) exit 0 ;;\n  stop) exit 0 ;;\n  container) case \"$2\" in inspect) printf 'running\\n' ;; stop) exit 0 ;; esac ;;\n  --help) printf 'help\\n' ;;\n  *) exit 0 ;;\nesac\n"
    }

    fn direct_manifest(provider: RuntimeProvider) -> CompositionManifest {
        let mut manifest = CompositionManifest::parse(include_str!("../../../examples/composition-v0.yaml")).unwrap();
        manifest.runtime.provider = provider;
        manifest.runtime.capability = "runtime.direct_container".to_owned();
        manifest.runtime.wsl_distribution = None;
        manifest.providers.clear();
        manifest.providers.insert(
            "service-lifecycle".to_owned(),
            ProviderDeclaration {
                capability: SERVICE_LIFECYCLE_CAPABILITY.to_owned(),
                status: ProviderStatus::Available,
                implementation: Some("container-cli-direct".to_owned()),
            },
        );
        manifest.artifacts.clear();
        let service = manifest.services.get_mut("worker").unwrap();
        service.resources = None;
        service.health_check = None;
        manifest
    }

    fn exercise_direct_runtime(
        provider: RuntimeProvider,
        command_name: &str,
        expected_inspect: &str,
        expected_stop: &[&str],
    ) {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(directory.path(), command_name, direct_runtime_script());
        let _environment = install_fake_command_path(directory.path(), &log);
        let service = direct_service();
        let runtime = ContainerRuntime::for_service(&provider, &None, "run", "worker", &service).unwrap();
        let id = runtime.spawn(&ImageRef::new(&service.image)).unwrap();
        assert_eq!(id.as_ref(), "fake-id");
        assert_eq!(
            fs::read_to_string(&log).unwrap(),
            "run\n-d\n--name\nrun-worker\n--env\nMODE=test\nexample:latest\n/bin/worker\n"
        );
        assert_eq!(runtime.status(&id).unwrap(), ContainerStatus::Running);
        assert_eq!(fs::read_to_string(&log).unwrap(), expected_inspect);
        runtime.health(&id, &["true".to_owned()]).unwrap();
        assert_eq!(fs::read_to_string(&log).unwrap(), "exec\nfake-id\ntrue\n");
        runtime.stop(&id).unwrap();
        let expected = expected_stop.join("\n") + "\n";
        assert_eq!(fs::read_to_string(&log).unwrap(), expected);
    }

    #[test]
    fn direct_podman_command_preserves_arguments() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(
            directory.path(),
            "podman",
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PHENOCOMPOSE_PODMAN_TEST_LOG\"\nprintf 'ok\\n'\n",
        );
        let _environment = install_fake_command_path(directory.path(), &log);
        let arguments = vec!["version".to_owned(), "--format".to_owned(), "json".to_owned()];

        let output = podman_output_owned_with_timeout(&None, &arguments, Duration::from_secs(1)).unwrap();

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
        assert_eq!(fs::read_to_string(log).unwrap(), "version\n--format\njson\n");
    }

    #[test]
    fn capability_probe_receipt_records_actual_podman_command() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(
            directory.path(),
            "podman",
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PHENOCOMPOSE_PODMAN_TEST_LOG\"\nprintf '{\"version\":{\"Version\":\"5.8.4\"}}\\n'\n",
        );
        let _environment = install_fake_command_path(directory.path(), &log);

        let receipt = probe_runtime(&RuntimeProvider::Podman, &None).unwrap();

        assert!(receipt.ready);
        assert_eq!(receipt.executable, "podman");
        assert_eq!(
            receipt.commands,
            vec![vec![
                "podman".to_owned(),
                "info".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
            ]]
        );
        assert_eq!(receipt.version.as_deref(), Some("5.8.4"));
        assert_eq!(
            fs::read_to_string(log).unwrap(),
            "info\n--format\njson\n"
        );
    }

    #[test]
    fn wsl_podman_command_includes_distribution_route() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(
            directory.path(),
            "wsl.exe",
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$PHENOCOMPOSE_PODMAN_TEST_LOG\"\nprintf 'ok\\n'\n",
        );
        let _environment = install_fake_command_path(directory.path(), &log);
        let distribution = Some("podman-machine-default".to_owned());
        let arguments = vec!["image".to_owned(), "exists".to_owned(), "example:latest".to_owned()];

        let output = podman_output_owned_with_timeout(&distribution, &arguments, Duration::from_secs(1)).unwrap();

        assert!(output.status.success());
        assert_eq!(
            fs::read_to_string(log).unwrap(),
            "-d\npodman-machine-default\n--\npodman\nimage\nexists\nexample:latest\n"
        );
    }

    #[test]
    fn podman_command_timeout_kills_child_and_is_deterministic() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(directory.path(), "podman", "#!/bin/sh\nwhile :; do :; done\n");
        let _environment = install_fake_command_path(directory.path(), &log);

        let error = podman_output_owned_with_timeout(
            &None,
            &["version".to_owned()],
            Duration::from_millis(50),
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Backend);
        assert_eq!(error.code, "podman_timeout");
        assert_eq!(error.message, "Podman command timed out after 50 ms");
    }

    #[test]
    fn apple_containers_route_run_inspect_health_and_stop() {
        exercise_direct_runtime(
            RuntimeProvider::AppleContainers,
            "container",
            "inspect\nfake-id\n",
            &["stop\nfake-id"],
        );
    }

    #[test]
    fn wslc_routes_run_inspect_health_and_container_stop() {
        exercise_direct_runtime(
            RuntimeProvider::WslContainers,
            "wslc",
            "container\ninspect\nfake-id\n",
            &["container\nstop\nfake-id"],
        );
    }

    #[test]
    fn wslc_falls_back_to_container_executable_alias() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(directory.path(), "container.exe", direct_runtime_script());
        let _environment = install_fake_command_path(directory.path(), &log);

        let output = runtime_output_owned_with_timeout(
            ContainerBackend::WslContainers,
            &None,
            &["version".to_owned()],
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(output.status.success());
        assert_eq!(fs::read_to_string(log).unwrap(), "version\n");
    }

    #[test]
    fn direct_publisher_uses_provider_image_inspect() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let log = directory.path().join("args.log");
        write_fake_command(directory.path(), "container", direct_runtime_script());
        let _environment = install_fake_command_path(directory.path(), &log);
        let publisher = ContainerLocalPublisher::new(ContainerBackend::AppleContainers, &None);
        publisher.probe().unwrap();
        let artifact = ComposedArtifact::new("example:latest", ImageRef::new("example:latest"));
        let target = PublishTarget::new(publisher.target_kind(), "example:latest");
        let receipt = publisher.publish(&artifact, &target).unwrap();
        assert_eq!(receipt.published_at, "apple-containers-local://example:latest");
        assert_eq!(fs::read_to_string(log).unwrap(), "image\ninspect\nexample:latest\n");
    }

    #[test]
    fn apple_direct_apply_status_and_down_use_one_provider_route() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let command_directory = tempfile::tempdir().unwrap();
        let log = command_directory.path().join("args.log");
        write_fake_command(command_directory.path(), "container", direct_runtime_script());
        let _environment = install_fake_command_path(command_directory.path(), &log);
        let output = apply(direct_manifest(RuntimeProvider::AppleContainers), directory.path(), false).unwrap();
        assert_eq!(output.provider, "apple-containers");
        assert_eq!(output.containers, BTreeMap::from([("worker".to_owned(), "fake-id".to_owned())]));
        assert_eq!(status(directory.path(), &output.run_id).unwrap().services["worker"], "running");
        assert_eq!(down(directory.path(), &output.run_id).unwrap().lifecycle, RunLifecycle::Down);
    }

    #[test]
    fn wslc_direct_apply_status_and_down_use_container_subcommands() {
        let _lock = environment_lock();
        let directory = tempfile::tempdir().unwrap();
        let command_directory = tempfile::tempdir().unwrap();
        let log = command_directory.path().join("args.log");
        write_fake_command(command_directory.path(), "wslc", direct_runtime_script());
        let _environment = install_fake_command_path(command_directory.path(), &log);
        let output = apply(direct_manifest(RuntimeProvider::WslContainers), directory.path(), false).unwrap();
        assert_eq!(output.provider, "wsl-containers");
        assert_eq!(output.containers, BTreeMap::from([("worker".to_owned(), "fake-id".to_owned())]));
        assert_eq!(status(directory.path(), &output.run_id).unwrap().services["worker"], "running");
        assert_eq!(down(directory.path(), &output.run_id).unwrap().lifecycle, RunLifecycle::Down);
        assert_eq!(fs::read_to_string(log).unwrap(), "container\nstop\nfake-id\n");
    }
}
