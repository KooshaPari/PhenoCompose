# PhenoCompose Config

Layered configuration: defaults < file < env vars < CLI flags.

## Stack

- `figment` - layered config with TOML + env + CLI sources
- `serde` - deserialize to typed config structs

## Layers (lowest -> highest priority)

1. **Built-in defaults** (in `pheno-config` crate)
2. **`pheno-compose.toml`** (file at known paths)
3. **Environment variables** (prefix `PHENOCOMPOSE_`)
4. **CLI flags** (via `clap`)

## Example

```toml
# pheno-compose.toml
[daemon]
listen = "127.0.0.1:20128"
workers = 4

[storage]
backend = "sqlite"
path = "/var/lib/phenocompose/state.db"

[compose]
default_driver = "apple-container"
timeout_seconds = 300
```

```bash
# Override via env (highest non-CLI priority)
export PHENOCOMPOSE_DAEMON__LISTEN="0.0.0.0:20128"
export PHENOCOMPOSE_STORAGE__BACKEND="redis"

# Override via CLI
phenocompose --config /etc/phenocompose.toml daemon serve \
  --daemon.listen 0.0.0.0:8080
```

## Config struct pattern

```rust
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Env}};

#[derive(Deserialize)]
pub struct PhenoConfig {
    pub daemon: DaemonConfig,
    pub storage: StorageConfig,
    pub compose: ComposeConfig,
}

pub fn load() -> Result<PhenoConfig, figment::Error> {
    Figment::new()
        .merge(Toml::file("pheno-compose.toml"))
        .merge(Env::prefixed("PHENOCOMPOSE_").split("__"))
        .extract()
}
```

## Validation

- Use `validator` crate for field-level constraints
- Reject invalid config at startup (fail fast)
- Log effective config (with redaction) at startup
