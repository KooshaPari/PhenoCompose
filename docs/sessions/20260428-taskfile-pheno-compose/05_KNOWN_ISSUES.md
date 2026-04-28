# Known Issues

- The repo has no top-level lint script for the docs/TypeScript side, so `lint` focuses on the manifest-backed Go and Rust checks plus the docs build trigger.
- The root contains `Makefile.go`, which is not a buildable Go package, so the Go task commands must target `cmd/`, `internal/`, and `bindings/` explicitly instead of `./...`.
- The current Go tree still has pre-existing compile errors in `internal/domain/sandbox.go` and `internal/adapters/windows/windows.go`, so the Go build/test commands are not green yet.
- `pheno-compose-driver/Cargo.toml` currently fails `cargo metadata` because the `nvms` feature references a non-optional `nvms-ffi` dependency; common Rust tasks therefore only run manifests Cargo can parse.
- `clean` removes generated artifacts directly; it does not attempt a full dependency cache purge.
