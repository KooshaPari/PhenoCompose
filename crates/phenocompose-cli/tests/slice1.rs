use phenocompose_cli::model::{CompositionManifest, RuntimeProvider, API_VERSION};
use phenocompose_cli::{apply, render_plan, ErrorKind};

fn sample() -> CompositionManifest {
    CompositionManifest::parse(include_str!("../../../examples/composition-v0.yaml")).unwrap()
}

#[test]
fn sample_schema_v0_parses_and_validates() {
    let manifest = sample();
    assert_eq!(manifest.api_version, API_VERSION);
    assert_eq!(manifest.metadata.name, "phenotype-lab");
    assert_eq!(
        manifest.services["worker"]
            .resources
            .as_ref()
            .unwrap()
            .gpu
            .as_ref()
            .unwrap()
            .uuids[0],
        "GPU-123e4567-e89b-12d3-a456-426614174000"
    );
}

#[test]
fn rendering_and_digest_are_deterministic() {
    let manifest = sample();
    let first = render_plan(&manifest).unwrap();
    let second = render_plan(&manifest).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.manifest_sha256.len(), 64);

    let first_json = serde_json::to_string_pretty(&first).unwrap();
    let second_json = serde_json::to_string_pretty(&second).unwrap();
    assert_eq!(first_json.as_bytes(), second_json.as_bytes());
}

#[test]
fn unknown_top_level_and_nested_fields_are_rejected() {
    let input = include_str!("../../../examples/composition-v0.yaml");
    let top_level = format!("{input}\nunknown_field: true\n");
    let error = CompositionManifest::parse(&top_level).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(error.message.contains("unknown field"));

    let nested = input.replace(
        "    purpose: slice-1-contract",
        "    purpose: slice-1-contract\n  unknown_metadata_field: true",
    );
    let error = CompositionManifest::parse(&nested).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(error.message.contains("unknown field"));
}

#[test]
fn gpu_ordinal_selector_is_rejected() {
    let input =
        include_str!("../../../examples/composition-v0.yaml").replace("GPU-123e4567-e89b-12d3-a456-426614174000", "0");
    let error = CompositionManifest::parse(&input).unwrap_err();
    assert_eq!(error.code, "gpu_selector_invalid");
}

#[test]
fn mutating_apply_fails_closed_for_nvms() {
    let mut manifest = sample();
    manifest.runtime.provider = RuntimeProvider::Nvms;
    manifest.providers.clear();
    manifest.artifacts.clear();
    manifest.services.get_mut("worker").unwrap().health_check = None;
    let directory = tempfile::tempdir().unwrap();
    let error = apply(manifest, directory.path(), false).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Unsupported);
    assert_eq!(error.code, "nvms_persistence_unsupported");
    assert!(!directory.path().join("runs").exists());
}

#[test]
fn mutating_apply_fails_closed_for_placeholder_capabilities() {
    let directory = tempfile::tempdir().unwrap();
    let error = apply(sample(), directory.path(), false).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Unsupported);
    assert_eq!(error.code, "provider_unavailable");
}

#[test]
fn dry_run_does_not_require_provider_or_write_state() {
    let directory = tempfile::tempdir().unwrap();
    let output = apply(sample(), directory.path(), true).unwrap();
    assert!(output.dry_run);
    assert!(!output.mutation);
    assert!(output.containers.is_empty());
    assert!(directory.path().read_dir().unwrap().next().is_none());
}

#[test]
fn podman_contract_keeps_compatibility_token_explicit() {
    let value = serde_json::to_value(sample()).unwrap();
    assert_eq!(value["runtime"]["provider"], "podman");
    assert_eq!(
        value["runtime"]["portage_compatibility"]["external_engine_token"],
        "docker"
    );
    assert_eq!(value["runtime"]["portage_compatibility"]["effective_engine"], "podman");
    assert_eq!(
        value.to_string().matches("\"docker\"").count(),
        1,
        "docker is only an external Portage compatibility token"
    );
}

#[test]
fn unsupported_errors_are_machine_readable() {
    let directory = tempfile::tempdir().unwrap();
    let error = apply(sample(), directory.path(), false).unwrap_err();
    let json = serde_json::to_value(error.envelope()).unwrap();
    assert_eq!(json["error"]["kind"], "unsupported");
    assert!(json["error"]["capability"].is_string());
    assert!(json["error"]["provider"].is_string());
}
