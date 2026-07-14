# Cross-component foundation pilot

This pilot proves the boundary between PhenoCompose, BytePort, and NanoVMS without
moving cloud state into the composition or execution layers.

## Flow

1. Build a `Composition` with a DNS-compatible name and at least one service.
2. Render a declared `Target::Docker` or `Target::Kubernetes` plan.
3. Call `RenderedPlan::verify_digest()` before handing the plan to BytePort.
4. Pass `RenderedPlan::byteport_handoff()` to BytePort as `composition_digest` and
   `artifact_ref`; BytePort validates the digest and stores deployment ownership.
5. Render a declared `Target::NanoVms` plan separately.
6. Call `RenderedPlan::nanovms_handoff()` and pass its composition name and digest
   to NanoVMS `DeployWithPlan`; NanoVMS validates the immutable identity and records
   it on the sandbox.

For local/container execution, the same renderer can produce an explicit
`ExecutionHandoff` for Podman, Apple Containers, or the first-party WSL containers
extension. These backends consume Docker-format plans and remain runtime adapters;
they do not become BytePort providers or state stores.

## Invariants

- BytePort receives rendered artifact metadata, never NanoVMS credentials or runtime
  state.
- NanoVMS receives an immutable composition identity, never cloud provider state.
- A modified plan fails digest verification before either adapter is called.
- Target-specific handoffs reject the wrong renderer output.

The Rust unit test `handoffs_enforce_target_ownership` and the tamper-detection test
are the local contract gate. The Go adapter tests in BytePort and NanoVMS are the
language-boundary gates. BytePort exposes the owner-scoped desired-state endpoint
`POST /mesh/workloads`; its request body is the `MeshWorkloadIntent` fields plus
portable placement constraints. NanoVMS exposes `Engine::DeployComposition` for
the runtime half of the pilot. A client should submit the same digest to both
systems and compare BytePort's accepted intent with NanoVMS's correlation labels.
