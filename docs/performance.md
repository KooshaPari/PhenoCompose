# PhenoCompose Performance

Criterion-based benchmarks for the most-trafficked value types.

## Running benchmarks

```bash
cargo bench --manifest-path /Users/kooshapari/CodeProjects/PhenoCompose-clean/Cargo.toml -p phenocompose-benches
```

## Current benchmarks (L6)

| Bench | Purpose | Target |
|-------|---------|--------|
| `manifest_new` | measure `Manifest{}` literal allocation | < 50 ns |
| `manifest_clone` | measure Clone impl | < 100 ns |
| `secret_ref_locator` | measure `SecretRef::locator()` | < 5 ns |
| `port_error_display` | measure Display impl | < 50 ns |
| `secret_new` | measure `Secret::new()` | < 100 ns |

## Memory (L19)

Per-value-type RSS measurements and 0-alloc verification.
