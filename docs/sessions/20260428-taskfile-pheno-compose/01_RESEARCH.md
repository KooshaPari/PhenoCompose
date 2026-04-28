# Research

- Root manifests detected:
  - `go.mod` at the repository root.
  - `package.json` for VitePress docs.
  - `pheno-compose-driver/Cargo.toml` for the Rust driver crate.
- Existing guidance also points to Go as the primary implementation language, with Rust and TypeScript as supporting components.
- Existing repo commands:
  - Go: `go build ./...`, `go test ./...`, `golangci-lint run`
  - Rust: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`
  - Docs: `bun run docs:build` or `npm run docs:build`
