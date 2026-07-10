# PhenoCompose Testing Depth

Multi-layer test strategy: unit + property + integration + mutation + e2e.

## Layers

| Layer | Tooling | Scope | Target coverage |
|-------|---------|-------|-----------------|
| Unit | `cargo test` | function | 80%+ |
| Property | `proptest` | invariant | 5+ props per crate |
| Integration | `testcontainers` | adapter | 60%+ |
| Mutation | `cargo-mutants` | survives | 50%+ killed |
| E2E | `cucumber`/`trycmd` | CLI | 1+ per command |

## Property tests (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn manifest_roundtrip(name in "[a-zA-Z][a-zA-Z0-9_-]{0,31}") {
        let m = Manifest { name: name.clone(), artifact_name: None, tags: vec![] };
        let json = serde_json::to_string(&m).unwrap();
        let m2: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }
}
```

## Testcontainers (integration)

```rust
use testcontainers_modules::postgres;

#[tokio::test]
async fn secret_store_with_postgres() {
    let docker = clients::Cli::default();
    let pg = docker.run(postgres::Postgres::default());
    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", pg.get_host_port_ipv4(5432));
    let store = PgSecretStore::connect(&url).await.unwrap();
    // ... test cases
}
```

## Mutation testing (cargo-mutants)

```bash
cargo install cargo-mutants
cargo mutants --package phenocompose-port-types
```

Target: 50%+ mutants killed by tests. If lower, tests are too weak.

## CLI snapshot tests (trycmd)

```rust
#[test]
fn test_help_output() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.trycmd")
        .run("cargo run --bin phenocompose -- --help");
}
```

```
# tests/cmd/help.trycmd
$ phenocompose --help
?Manifest composition tool
?
?Usage: phenocompose <COMMAND>
?
?Commands:
?    compose    Compose a manifest
?    validate   Validate a manifest
```

## CI matrix

| Test | Fast | Required | Trigger |
|------|------|----------|---------|
| `cargo test` | yes | yes | every PR |
| `cargo test --release` | no | yes | every PR |
| `cargo mutants` | no | weekly | schedule |
| `cucumber` | no | yes | every PR |
| `cargo audit` | yes | yes | every PR |

## Coverage target

- 80% line coverage for all crates
- 100% coverage for `PortError` variants
- 100% coverage for security-critical paths
- Coverage drops in PRs fail CI
