# PhenoCompose Architecture (C4-style)

## System Context

```
+--------------------+         +--------------------+
|   CLI / API        | ----->  |  phenocompose-*    |
|   (cmd/, server)   |         |  crates (domain)   |
+--------------------+         +--------------------+
                                     |
                                     v
+--------------------+         +--------------------+
|   Adapters         | <-----  |   Port Traits     |
|   (apple, wsl,     |         |   (composer,       |
|   file, in-mem)    |         |   publisher,       |
+--------------------+         |   runtime,         |
                               |   secret_store)    |
                               +--------------------+
```

## Container View (rust crates)

```
pheno-compose-driver
  |-- phenocompose-port-composer
  |     |-- phenocompose-port-types (shared value types)
  |     +-- phenocompose-apple-container-adapter
  |     +-- phenocompose-wslc-adapter
  +-- phenocompose-port-publisher
  +-- phenocompose-port-runtime
  +-- phenocompose-port-secret
  |     +-- phenocompose-secret-file-adapter
  +-- phenocompose-port-di (DI container)
  +-- phenocompose-pheno-config (driver config)
```

## Component View (port-types module)

```
port-types/
  lib.rs         (re-exports + tests)
  compose.rs     (Manifest, ComposedArtifact, PublishTarget, Receipt)
  runtime.rs     (ImageRef, ContainerId, ContainerStatus)
  error.rs       (PortError)
  secret.rs      (SecretRef, Secret)
  oci/           (OCI image reference helpers - future split)
```

## ADR Index

- ADR-0001: Record architecture decisions
- ADR-0002: port-types decomposition
- ADR-0003: port trait design rules
