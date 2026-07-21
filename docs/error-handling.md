# PhenoCompose Error Handling

`thiserror` for libraries, structured error catalog, `Result<T, E>` everywhere.

## Library errors (thiserror)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("transport: {0}")]
    Transport(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}
```

## Application errors (anyhow)

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let path = std::env::var("PHENOCOMPOSE_CONFIG")?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading config from {}", path))?;
    let cfg: Config = toml::from_str(&raw)
        .with_context(|| format!("parsing config from {}", path))?;
    Ok(cfg)
}
```

## Error catalog

All port errors documented in `docs/error-catalog.md`:

| Variant | HTTP code | Retry? | User message |
|---------|-----------|--------|--------------|
| `Validation` | 400 | no | "Invalid input: <reason>" |
| `NotFound` | 404 | no | "<resource> not found" |
| `Transport` | 502 | yes | "Upstream service unavailable" |
| `Unsupported` | 501 | no | "Operation not supported" |

## Conversion (port -> http)

```rust
impl actix_web::ResponseError for PortError {
    fn status_code(&self) -> StatusCode { ... }
    fn error_response(&self) -> HttpResponse { ... }
}
```

## Tests

```rust
#[test]
fn test_validation_error_message() {
    let e = PortError::Validation("empty name".to_string());
    assert_eq!(e.to_string(), "validation: empty name");
}
```
