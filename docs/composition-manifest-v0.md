# Composition manifest V0

`phenocompose.dev/v0` is the first executable PhenoCompose contract. The Rust
model in `crates/phenocompose-cli/src/model.rs` is authoritative for Slice 1.
Every object rejects unknown fields.

The manifest represents:

- environment platform, toolkit version, and variables;
- a runtime capability and optional WSL distribution;
- provider capability declarations with `available`, `placeholder`, or
  `unsupported` status;
- named services, dependency order, commands, environment, CPU, memory, and
  GPU UUID constraints;
- health-check declarations, actions, artifacts, teardown order, and
  provenance requirements.

GPU selectors must be UUIDs (with an optional `GPU-` prefix). Device ordinals
such as `0` are invalid because they are host-order-dependent.

## Podman and Portage compatibility

Podman is the only effective container engine in this slice. A manifest may
contain the literal `docker` only in
`runtime.portage_compatibility.external_engine_token`, where it records an
external Portage protocol token. The adjacent `effective_engine` is fixed to
`podman`; the token never selects a deployment engine.

On Windows, setting `runtime.wsl_distribution` routes commands through:

```text
wsl.exe -d <distribution> -- podman ...
```

## Command truthfulness

- `plan` parses, validates, canonicalizes JSON object keys, and hashes the
  compact canonical manifest with SHA-256.
- `apply --dry-run` returns the run identity without probing providers,
  starting containers, or writing state.
- Mutating `apply` rejects placeholder/unsupported providers and unsupported
  requested features before probing Podman.
- Mutating Podman apply uses a prebuilt-image Composer, a local-image
  Publisher check, and a Podman Runtime adapter. Successful container IDs are
  persisted under `.phenocompose/runs`.
- `status`, `down`, and `export-provenance` require persisted successful run
  state.
- `run-action` validates the persisted action and then returns a structured
  unsupported error because the current Runtime port has no exec method.
- NVMS mutation is rejected because the existing driver destroys instances on
  drop and has no API to reattach a persisted instance ID.

Errors are JSON objects with stable `kind` and `code` fields. Unsupported
errors also identify `capability` and `provider`.
