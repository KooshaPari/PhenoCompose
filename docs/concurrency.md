# PhenoCompose Concurrency

Structured concurrency for parallel composition + port dispatch.

## Stack (Rust)

- `tokio` (async runtime, v1)
- `rayon` (data-parallel)
- `loom` (model checking for concurrency)
- `tokio-util` (cancellation, semaphores)

## Patterns

### Bounded parallel composition

```rust
use rayon::prelude::*;
use phenocompose_port_composer::Composer;

let manifests: Vec<Manifest> = ...;
let results: Vec<Result<ComposedArtifact, _>> = manifests
    .par_iter()
    .map(|m| composer.compose(m))
    .collect();
```

### Structured cancellation

```rust
use tokio::select;

async fn resolve_with_timeout(req: ResolveRequest) -> Result<Resolved> {
    select! {
        result = resolver.resolve(&req) => result,
        _ = tokio::time::sleep(Duration::from_secs(5)) => Err(Timeout),
    }
}
```

### Bounded concurrency (semaphore)

```rust
use tokio::sync::Semaphore;
let permits = Arc::new(Semaphore::new(32));
for req in requests {
    let permit = permits.clone().acquire_owned().await?;
    tokio::spawn(async move {
        handle(req).await;
        drop(permit);
    });
}
```

## Model testing (loom)

```rust
use loom::sync::Arc;
use loom::thread;

#[test]
fn test_concurrent_state() {
    loom::model(|| {
        let state = Arc::new(State::new());
        let s1 = state.clone();
        let s2 = state.clone();
        let h1 = thread::spawn(move || s1.update());
        let h2 = thread::spawn(move || s2.read());
        h1.join().unwrap();
        h2.join().unwrap();
    });
}
```

## Bench

| Pattern | Target |
|---------|--------|
| 100 parallel compose | < 2s |
| 1000 parallel compose (rayon) | < 10s |
| Cancellation latency | < 10ms |
| Semaphore acquisition | < 1us |
