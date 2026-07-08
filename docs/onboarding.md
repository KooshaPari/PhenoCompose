# PhenoCompose Onboarding — 5-Minute Quickstart

## Prerequisites

- Rust 1.75+
- Cargo (bundled with Rust)
- Git

## Clone and build

```bash
git clone https://github.com/KooshaPari/PhenoCompose
cd PhenoCompose
./scripts/dev-bootstrap.sh
cargo build --workspace
```

## Run the tests

```bash
cargo test --workspace
```

## Try the agentctl CLI

```bash
echo '{"method":"composer.compose","params":{"manifest":{"name":"hello"}}}}' | cargo run -p phenocompose-agentctl
```

## Run the example adapter

```bash
cargo run -p phenocompose-in-memory
```

## Next steps

- Read `docs/architecture.md` for the system overview
- Read `docs/adr/` for design decisions
- Read `docs/monitoring.md` for observability
- See `examples/` for sample adapters
