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

## Service graph and lifecycle planning

Services form a directed acyclic graph through `depends_on`. Planning and
apply use a deterministic topological sort (lexicographic tie-break on service
names). Cycles fail closed with `dependency_cycle`.

`apply --dry-run` emits a `lifecycle` object (`phenocompose.lifecycle/v0`)
containing:

- `order`: topologically sorted service names;
- `intents`: alternating `create` and `start` phases per service, including
  declared `depends_on` edges and health-check retry bounds on `start`.

Dry-run does not probe Podman, WSL, or NanoVMS and does not write run state.

Mutating apply requires `runtime.service_lifecycle` declared with
`status: available`. Until the NanoVMS Podman lifecycle bridge is wired, keep
this provider at `placeholder` so apply fails closed with
`service_lifecycle_placeholder` rather than inventing green status.

When mutating apply succeeds in a future slice, mid-graph failures roll back
only containers recorded in the current run's `RollbackContract` (reverse stop
order). Health-check readiness enforcement and bounded retries remain gated on
the same lifecycle capability because the current Runtime port exposes spawn,
stop, and status only.

Example four-service dual-GPU stack:
`examples/dual-gpu-inference-v0.yaml` (SGLang + llama-primary + llama-helper +
pheno-serve). Harness dogfood can reference
`examples/harness-dual-gpu-manifest-stub.yaml` without copying harness paths
into this repo.

## GPU selectors

GPU selectors must be UUIDs (with an optional `GPU-` prefix). Device ordinals
such as `0` are invalid because they are host-order-dependent.

For `run-action`, NanoVMS evaluation requests carry GPU identity as UUID plus
CUDA toolkit version only. Unverified fields (`name`, `architecture`,
`compute_capability`) are omitted from outbound `resource_manifest.gpus`; the
CLI does not synthesize hardware claims from manifest text.

Per-service CUDA semantics (for example CUDA 13 on Ampere SGLang vs CUDA 12
transitional llama.cpp builds) are expressed through service `environment`
entries; the manifest environment toolkit remains the closure default for
evaluation actions.

## Actions and output roots

Each action names a target service, command argv, and `output_root`. The path
must be workspace-relative or fully absolute; drive-relative paths, empty
values, and parent traversal are rejected. Relative roots resolve against the
process working directory at invocation time and affect the plan digest.

`run-action` requires a persisted successful Podman apply with the exact
historical docker-schema route (`runtime.provider: podman` plus
`portage_compatibility.external_engine_token: docker` and
`effective_engine: podman`). It executes through the bounded NanoVMS
evaluation boundary (subprocess JSON protocol), not Runtime container exec.

Job provenance records the resolved absolute `output_root` plus NanoVMS-reported
`output_root_created` (bool) and `output_root_available_bytes` (optional u64).
These fields export through `export-provenance` and reject unknown keys.

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

## NanoVMS Podman lifecycle gate (remaining work)

True four-endpoint mutating apply for the dual-GPU inference stack remains
blocked until all of the following are available:

1. `runtime.service_lifecycle` provider attested `available` (NanoVMS Podman
   bridge with dependency-aware readiness, not raw CLI process launch inside
   PhenoCompose).
2. Runtime port extensions for health-check polling and bounded retries (or
   delegated attestation from NanoVMS lifecycle evidence).
3. Pre-published OCI images present in Podman local storage for all four
   services (`lmsysorg/sglang`, llama.cpp CUDA server, `pheno-serve`).
4. GPU CDI device injection verified for both UUIDs through the WSL Podman
   pipe without Docker Desktop.

PhenoCompose intentionally does not fake success for any of the above. Slice 1
Podman apply remains available only for manifests without health checks once
`runtime.service_lifecycle` is marked available and other providers pass.

## Command truthfulness

- `plan` parses, validates, canonicalizes JSON object keys, and hashes the
  compact canonical manifest with SHA-256.
- `apply --dry-run` returns run identity plus ordered lifecycle intents
  without probing providers, starting containers, or writing state.
- Mutating `apply` rejects placeholder/unsupported providers (except the
  dedicated lifecycle gate), missing `runtime.service_lifecycle`, and
  unsupported requested features before probing Podman.
- Mutating Podman apply uses a prebuilt-image Composer, a local-image
  Publisher check, and a Podman Runtime adapter. Successful container IDs are
  persisted under `.phenocompose/runs`.
- `status`, `down`, and `export-provenance` require persisted successful run
  state.
- `run-action` validates route, GPU closure, and `output_root`, invokes
  NanoVMS, validates attested provenance and lifecycle bounds, and persists
  job provenance under `.phenocompose/runs`.
- NVMS runtime `apply` mutation is rejected because the existing driver
  destroys instances on drop and has no API to reattach a persisted instance ID.

Errors are JSON objects with stable `kind` and `code` fields. Unsupported
errors also identify `capability` and `provider`.
