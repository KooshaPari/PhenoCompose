# Unified Composition Contract

**Status:** proposed implementation contract

## Scope

PhenoCompose accepts one service-oriented composition definition and produces
validated plans for four execution targets:

- `docker` — local or OCI-compatible container execution;
- `kubernetes` — Kubernetes workload/service resources;
- `process` — host-process supervision;
- `nanovms` — execution through the NanoVMS engine abstraction.

BytePort consumes the resulting **IaC handoff**, not the source manifest. It
owns provider selection, cloud state, deployment credentials, and apply/status
operations. NanoVMS consumes the NanoVMS plan and owns engine selection and
instance lifecycle. PhenoCompose never applies cloud state itself.

## Canonical model

```yaml
apiVersion: phenocompose/v1alpha1
kind: Composition
metadata:
  name: example-api
  labels:
    environment: staging
spec:
  services:
    api:
      image: ghcr.io/example/api@sha256:...
      command: ["/app/api"]
      environment:
        PORT: "8080"
      ports:
        - name: http
          containerPort: 8080
          protocol: tcp
      health:
        type: http
        path: /healthz
        port: 8080
      resources:
        cpu: "500m"
        memory: "512Mi"
      dependsOn: []
  targets:
    docker: {}
    kubernetes:
      namespace: example
    process: {}
    nanovms:
      tier: wasm
```

Required invariants:

1. `metadata.name` and every service name are non-empty DNS-label-compatible
   identifiers.
2. A service has an image or a command; a target renderer rejects unsupported
   combinations rather than silently dropping them.
3. `dependsOn` references existing services and is acyclic.
4. Each named port is unique per service and has a port in `1..=65535`.
5. Secret values are references only; resolved secret values never appear in a
   rendered plan, logs, or BytePort handoff.
6. Rendering is deterministic: identical normalized input and target yield
   byte-identical plan content and a stable digest.

## Renderer output

Every renderer returns the same envelope:

```text
RenderedPlan {
  target: Docker | Kubernetes | Process | NanoVms,
  composition_name: String,
  digest: sha256:<normalized-plan>,
  content_type: String,
  content: bytes,
  resources: [ResourceRef],
  diagnostics: [Diagnostic]
}
```

Target content types are:

| Target | Content type | Required result |
| --- | --- | --- |
| Docker | `application/x-yaml;type=docker-compose` | Compose file with services, ports, environment references, health checks, and dependency ordering |
| Kubernetes | `application/x-yaml;type=kubernetes` | Deployment/StatefulSet, Service where ports exist, resource limits, probes, namespace metadata |
| Process | `application/x-yaml;type=process-compose` | process-compose document with commands, environment references, readiness, and dependencies |
| NanoVMS | `application/json;type=nanovms-plan` | Per-service NanoVMS instance requests with tier, image/command, resources, network, and health contract |

## BytePort handoff

`BytePortIaCRequest` contains only:

- composition name and plan digest;
- a selected cloud target and region/account reference;
- the Kubernetes/Docker/provider artifact reference produced by PhenoCompose;
- declared resource and health requirements;
- opaque secret references.

BytePort must reject a handoff when its digest does not match the rendered
content, when a secret value is supplied instead of a reference, or when the
target has not been validated by PhenoCompose.

## NanoVMS handoff

`NanoVmsPlan` contains only an execution request. NanoVMS selects a compatible
engine according to the requested tier and host capability, returning a stable
instance id, selected engine, lifecycle state, and failure code. It must not
receive cloud credentials or mutate BytePort state.

## Acceptance tests

1. One fixture renders successfully to all four targets and preserves service
   names, dependency order, ports, health checks, and resource limits.
2. Invalid names, cycles, invalid ports, missing executable fields, and raw
   secret values fail before any renderer or runtime is called.
3. The same fixture rendered twice yields the same digest for every target.
4. A BytePort handoff accepts a matching, validated plan and rejects a changed
   plan or secret value.
5. A NanoVMS handoff starts and stops a supported Linux test instance and
   reports the selected engine and health state.
