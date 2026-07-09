# PhenoCompose Migration Guide

## 0.1.x -> 0.2.x

### Breaking changes

- `Manifest::artifact_name` is now optional; use `Option<String>`
- `Secret::new()` now requires `SecretRef`, not a string

### Migration steps

```rust
// Before
let secret = Secret::new("db-password".to_string(), "hunter2".to_string());

// After
use phenocompose_port_types::{Secret, SecretRef};
let secret = Secret::new(SecretRef::new("db-password"), "hunter2");
```

### Editor codemod

```bash
cargo install cargo-fix
cargo fix --edition-idioms
```

## Support window

- Latest 2 minor versions supported
- Security patches for latest only
