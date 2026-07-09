# PhenoCompose Logging

Structured logging via `tracing` + `tracing-subscriber`.

## Stack

- `tracing` - structured events with spans
- `tracing-subscriber` - JSON sink for production, pretty for dev
- `tracing-bunyan-formatter` - alternative bunyan format

## Setup

```rust
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,phenocompose=debug"));
    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(false)
        .with_span_list(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
```

## Usage

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(cfg), fields(sandbox_id = %cfg.name))]
pub async fn create_sandbox(cfg: SandboxConfig) -> Result<Sandbox, Error> {
    info!("creating sandbox");
    let sb = adapter.create(cfg).await?;
    info!(id = %sb.id, "sandbox created");
    Ok(sb)
}
```

## Redaction

```rust
use tracing::field;

#[instrument(fields(token = field::display("REDACTED")))]
pub async fn auth(token: &str) -> Result<Session, Error> { ... }
```

## Log levels

- `ERROR` - 4xx/5xx responses, panics
- `WARN` - retried operations, deprecated API use
- `INFO` - lifecycle events (start, stop, sandbox created)
- `DEBUG` - per-request details
- `TRACE` - per-line execution

## Sampling (high-traffic paths)

```rust
use tracing_subscriber::filter::LevelFilter;

let sampling = LevelFilter::INFO
    .with_target("phenocompose::sandbox::hot_path", LevelFilter::WARN);
```
