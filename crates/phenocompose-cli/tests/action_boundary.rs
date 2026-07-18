use phenocompose_cli::model::{CompositionManifest, GpuRequirements, GpuVendor, ResourceRequirements, Service};
use phenocompose_cli::nvms::{
    EvaluationClient, EvaluationProvenance, EvaluationRequest, EvaluationResult, Lifecycle, ACTION_VERSION,
};
use phenocompose_cli::{
    export_provenance, load_job_provenance, run_action_with_client, run_action_with_client_at, CliError, RunLifecycle,
    RunState,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

const RUN_ID: &str = "slice1-run";
const UUID_A: &str = "GPU-123e4567-e89b-12d3-a456-426614174000";
const UUID_B: &str = "GPU-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

struct FakeClient {
    result: Mutex<Option<Result<EvaluationResult, CliError>>>,
    request: Mutex<Option<EvaluationRequest>>,
}

impl FakeClient {
    fn result(result: EvaluationResult) -> Self {
        Self {
            result: Mutex::new(Some(Ok(result))),
            request: Mutex::new(None),
        }
    }

    fn error(code: &str) -> Self {
        Self {
            result: Mutex::new(Some(Err(CliError::backend(code, "injected client failure")))),
            request: Mutex::new(None),
        }
    }
}

impl EvaluationClient for FakeClient {
    fn execute(&self, request: &EvaluationRequest) -> phenocompose_cli::Result<EvaluationResult> {
        *self.request.lock().unwrap() = Some(request.clone());
        self.result.lock().unwrap().take().unwrap()
    }
}

fn setup(mut edit: impl FnMut(&mut CompositionManifest)) -> (TempDir, RunState) {
    let mut manifest = CompositionManifest::parse(include_str!("../../../examples/composition-v0.yaml")).unwrap();
    manifest.environment.toolkit.version = "12.8".to_owned();
    manifest.providers.clear();
    manifest.artifacts.clear();
    manifest.services.get_mut("worker").unwrap().health_check = None;
    edit(&mut manifest);
    manifest.validate().unwrap();
    let digest = manifest.plan().unwrap().manifest_sha256;
    let state = RunState {
        state_version: "phenocompose.run/v0".to_owned(),
        run_id: RUN_ID.to_owned(),
        manifest_sha256: digest,
        provider: "podman".to_owned(),
        created_unix_seconds: 1,
        lifecycle: RunLifecycle::Running,
        containers: BTreeMap::from([("worker".to_owned(), "container-id".to_owned())]),
        manifest,
    };
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join(format!("{RUN_ID}.json")),
        serde_json::to_vec_pretty(&state).unwrap(),
    )
    .unwrap();
    (directory, state)
}

fn success_result(state: &RunState, uuids: Vec<String>) -> EvaluationResult {
    let stdout = "ok".to_owned();
    let stderr = "note".to_owned();
    EvaluationResult {
        version: ACTION_VERSION.to_owned(),
        success: true,
        error_code: String::new(),
        error_message: String::new(),
        lifecycle: Lifecycle {
            exit_code: 0,
            duration_ms: 25,
            timed_out: false,
            truncated: false,
            stdout_sha256: hash(&stdout),
            stderr_sha256: hash(&stderr),
            stdout,
            stderr,
        },
        provenance: EvaluationProvenance {
            manifest_sha256: state.manifest_sha256.clone(),
            effective_engine: "podman".to_owned(),
            resolved_provider: "podman".to_owned(),
            execution_plane: "nanovms".to_owned(),
            podman_pipe: "npipe:////./pipe/podman-machine-default".to_owned(),
            gpu_uuids: uuids,
            job_directory: "job-output".to_owned(),
        },
        released: true,
    }
}

fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn gpu_service(image: &str, uuid: &str) -> Service {
    Service {
        image: image.to_owned(),
        depends_on: Vec::new(),
        command: Vec::new(),
        environment: BTreeMap::new(),
        resources: Some(ResourceRequirements {
            cpu_millis: None,
            memory_bytes: None,
            gpu: Some(GpuRequirements {
                vendor: GpuVendor::Nvidia,
                uuids: vec![uuid.to_owned()],
            }),
        }),
        health_check: None,
    }
}

#[test]
fn maps_action_and_transitive_gpu_closure_without_runtime_commands() {
    let (directory, state) = setup(|manifest| {
        manifest
            .services
            .insert("base".to_owned(), gpu_service("base:image", UUID_B));
        manifest
            .services
            .get_mut("worker")
            .unwrap()
            .depends_on
            .push("base".to_owned());
    });
    let expected = vec![UUID_A.to_owned(), UUID_B.to_owned()];
    let client = FakeClient::result(success_result(&state, expected.clone()));
    let job = run_action_with_client(directory.path(), RUN_ID, "inspect-worker", "mapping", &client).unwrap();
    let request = client.request.lock().unwrap().clone().unwrap();

    assert_eq!(request.executable, "/usr/bin/env");
    assert!(request.argv.is_empty());
    assert_eq!(request.backend, "podman");
    assert!(request.fallback_backends.is_empty());
    assert_eq!(
        request
            .gpu_bindings
            .iter()
            .map(|binding| binding.uuid.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(request
        .gpu_bindings
        .iter()
        .all(|binding| binding.cuda_toolkit == "12.8"));
    assert_eq!(job.dependency_services, vec!["base", "worker"]);
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(!serialized.contains("\"podman run\""));
    assert!(!serialized.contains("\"wsl.exe\""));
}

#[test]
fn resolves_relative_output_root_against_workspace_and_serializes_exactly() {
    let (directory, state) = setup(|_| {});
    let workspace = tempfile::tempdir().unwrap();
    let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));

    run_action_with_client_at(
        workspace.path(),
        directory.path(),
        RUN_ID,
        "inspect-worker",
        "relative-root",
        &client,
    )
    .unwrap();

    let request = client.request.lock().unwrap().clone().unwrap();
    let expected = workspace.path().join("jobs").join("harbor");
    assert_eq!(request.output_root, expected.to_str().unwrap());
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["output_root"], expected.to_str().unwrap());
    assert!(!directory.path().join(format!("{RUN_ID}.outputs")).exists());
}

#[test]
fn accepts_fully_absolute_output_root_without_rebasing() {
    let workspace = tempfile::tempdir().unwrap();
    let absolute = workspace.path().join("absolute-output");
    let (directory, state) = setup(|manifest| {
        manifest.actions.get_mut("inspect-worker").unwrap().output_root = Some(absolute.to_str().unwrap().to_owned());
    });
    let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));

    run_action_with_client_at(
        workspace.path(),
        directory.path(),
        RUN_ID,
        "inspect-worker",
        "absolute-root",
        &client,
    )
    .unwrap();

    assert_eq!(
        client.request.lock().unwrap().as_ref().unwrap().output_root,
        absolute.to_str().unwrap()
    );
}

#[test]
fn rejects_output_root_traversal_before_calling_client() {
    for (job_id, root, code) in [
        ("parent", "jobs/../escape", "action_output_root_traversal"),
        ("missing", "", "action_output_root_missing"),
    ] {
        let (directory, state) = setup(|manifest| {
            manifest.actions.get_mut("inspect-worker").unwrap().output_root =
                (!root.is_empty()).then(|| root.to_owned());
        });
        let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));
        let error = run_action_with_client_at(
            directory.path(),
            directory.path(),
            RUN_ID,
            "inspect-worker",
            job_id,
            &client,
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(client.request.lock().unwrap().is_none());
    }
}

#[cfg(windows)]
#[test]
fn rejects_ambiguous_windows_output_roots() {
    for (job_id, root) in [("drive-relative", r"C:jobs\harbor"), ("root-relative", r"\jobs\harbor")] {
        let (directory, state) = setup(|manifest| {
            manifest.actions.get_mut("inspect-worker").unwrap().output_root = Some(root.to_owned());
        });
        let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));
        let error = run_action_with_client_at(
            directory.path(),
            directory.path(),
            RUN_ID,
            "inspect-worker",
            job_id,
            &client,
        )
        .unwrap_err();
        assert_eq!(error.code, "action_output_root_ambiguous");
        assert!(client.request.lock().unwrap().is_none());
    }
}

#[test]
fn output_root_changes_plan_digest_deterministically() {
    let manifest = CompositionManifest::parse(include_str!("../../../examples/composition-v0.yaml")).unwrap();
    let first = manifest.plan().unwrap();
    assert_eq!(first, manifest.plan().unwrap());

    let mut changed = manifest;
    changed.actions.get_mut("inspect-worker").unwrap().output_root = Some("jobs/other".to_owned());
    let changed_plan = changed.plan().unwrap();

    assert_ne!(first.manifest_sha256, changed_plan.manifest_sha256);
    assert_eq!(changed_plan, changed.plan().unwrap());
}

#[test]
fn rejects_unsafe_job_ids_before_calling_client() {
    let (directory, state) = setup(|_| {});
    let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));
    for (index, value) in ["../escape", "has/slash", "", ".hidden", "white space"]
        .into_iter()
        .enumerate()
    {
        let error = run_action_with_client(directory.path(), RUN_ID, "inspect-worker", value, &client).unwrap_err();
        assert_eq!(error.code, "job_id_invalid", "case {index}");
    }
    assert!(client.request.lock().unwrap().is_none());
}

#[test]
fn rejects_missing_toolkit_for_gpu_dependency() {
    let (directory, state) = setup(|manifest| manifest.environment.toolkit.version.clear());
    let client = FakeClient::result(success_result(&state, vec![UUID_A.to_owned()]));
    let error =
        run_action_with_client(directory.path(), RUN_ID, "inspect-worker", "missing-toolkit", &client).unwrap_err();
    assert_eq!(error.code, "toolkit_missing");
    assert!(client.request.lock().unwrap().is_none());
}

#[test]
fn preserves_malformed_and_nonzero_client_failures() {
    for code in ["nvms_malformed_response", "nvms_request_rejected"] {
        let (directory, _) = setup(|_| {});
        let error = run_action_with_client(
            directory.path(),
            RUN_ID,
            "inspect-worker",
            code,
            &FakeClient::error(code),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        let job = load_job_provenance(directory.path(), RUN_ID, code).unwrap();
        assert!(!job.success);
        assert_eq!(job.error_code, code);
    }
}

#[test]
fn rejects_digest_route_and_binding_mismatches() {
    for (job_id, mutate, expected_code) in [
        (
            "digest",
            (|result: &mut EvaluationResult| result.provenance.manifest_sha256 = "0".repeat(64))
                as fn(&mut EvaluationResult),
            "nvms_manifest_mismatch",
        ),
        (
            "engine",
            |result: &mut EvaluationResult| result.provenance.effective_engine = "docker".to_owned(),
            "nvms_route_mismatch",
        ),
        (
            "provider",
            |result: &mut EvaluationResult| result.provenance.resolved_provider = "gvisor".to_owned(),
            "nvms_route_mismatch",
        ),
        (
            "binding",
            |result: &mut EvaluationResult| result.provenance.gpu_uuids.clear(),
            "nvms_gpu_binding_mismatch",
        ),
    ] {
        let (directory, state) = setup(|_| {});
        let mut result = success_result(&state, vec![UUID_A.to_owned()]);
        mutate(&mut result);
        let error = run_action_with_client(
            directory.path(),
            RUN_ID,
            "inspect-worker",
            job_id,
            &FakeClient::result(result),
        )
        .unwrap_err();
        assert_eq!(error.code, expected_code);
        assert!(!load_job_provenance(directory.path(), RUN_ID, job_id).unwrap().success);
    }
}

#[test]
fn preserves_timed_out_lifecycle_and_hashes() {
    let (directory, state) = setup(|_| {});
    let mut result = success_result(&state, vec![UUID_A.to_owned()]);
    result.success = false;
    result.released = true;
    result.error_code = "action_timeout".to_owned();
    result.error_message = "deadline exceeded".to_owned();
    result.lifecycle.exit_code = -1;
    result.lifecycle.timed_out = true;
    let error = run_action_with_client(
        directory.path(),
        RUN_ID,
        "inspect-worker",
        "timeout",
        &FakeClient::result(result),
    )
    .unwrap_err();
    assert_eq!(error.code, "action_timeout");
    let job = load_job_provenance(directory.path(), RUN_ID, "timeout").unwrap();
    let lifecycle = job.lifecycle.unwrap();
    assert!(lifecycle.timed_out);
    assert_eq!(lifecycle.stdout_sha256, hash(&lifecycle.stdout));
    assert_eq!(lifecycle.stderr_sha256, hash(&lifecycle.stderr));
}

#[test]
fn rejects_lifecycle_limit_and_hash_mismatch() {
    for (job_id, mutate, expected) in [
        (
            "limit",
            (|result: &mut EvaluationResult| result.lifecycle.duration_ms = 300_001) as fn(&mut EvaluationResult),
            "nvms_lifecycle_limits",
        ),
        (
            "hash",
            |result: &mut EvaluationResult| result.lifecycle.stdout_sha256 = "0".repeat(64),
            "nvms_evidence_hash_mismatch",
        ),
    ] {
        let (directory, state) = setup(|_| {});
        let mut result = success_result(&state, vec![UUID_A.to_owned()]);
        mutate(&mut result);
        let error = run_action_with_client(
            directory.path(),
            RUN_ID,
            "inspect-worker",
            job_id,
            &FakeClient::result(result),
        )
        .unwrap_err();
        assert_eq!(error.code, expected);
    }
}

#[test]
fn successful_provenance_is_atomic_and_exported() {
    let (directory, state) = setup(|_| {});
    let job = run_action_with_client(
        directory.path(),
        RUN_ID,
        "inspect-worker",
        "success",
        &FakeClient::result(success_result(&state, vec![UUID_A.to_owned()])),
    )
    .unwrap();
    assert!(job.success);
    let jobs_dir = directory.path().join(format!("{RUN_ID}.jobs"));
    assert!(jobs_dir.join("success.json").is_file());
    assert!(fs::read_dir(&jobs_dir).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));

    let exported = export_provenance(directory.path(), RUN_ID).unwrap();
    assert_eq!(exported.jobs["success"], job);
}
