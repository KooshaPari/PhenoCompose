# PhenoCompose Dev Loop

## Toolchain

- Rust 1.75+ (managed via mise or rustup)
- Go 1.21+ (for nanovms interop)
- mise: `mise.toml` pins all versions

## Container

`.devcontainer/devcontainer.json` provides:
- rust:1.75-bookworm base
- go 1.21 feature
- cargo-nextest, cargo-deny, release-plz pre-installed
- Port 20128 forwarded (daemon)

## Common commands

```bash
# Build everything
cargo build --workspace

# Run all tests
cargo nextest run --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Fuzz
cargo fuzz --manifest-path fuzz run manifest_parse
```

## Hot-reload

```bash
cargo install cargo-watch
cargo watch -x 'nextest run -p phenocompose-port-types'
```
