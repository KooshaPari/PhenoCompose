# PhenoCompose Event-Driven

Event bus for cross-component signaling + outbox pattern for reliable delivery.

## Stack

- `tokio::sync::broadcast` - in-process pub/sub
- `NATS` (optional) - cross-process pub/sub
- `outbox` pattern - durable event log

## In-process events

```rust
use phenocompose_eventbus::{EventBus, Event};

let bus = EventBus::new(1024);
let mut rx = bus.subscribe::<ComposedEvent>();

bus.publish(ComposedEvent { id, manifest }).await;
while let Some(event) = rx.recv().await {
    println!("composed: {}", event.id);
}
```

## Event types

```rust
#[derive(Event, Clone, Debug)]
pub enum PhenoEvent {
    Composed(ComposedEvent),
    Published(PublishedEvent),
    SandboxStarted(SandboxStartedEvent),
    SecretRotated(SecretRotatedEvent),
}
```

## Outbox pattern

For durable, exactly-once delivery to external systems:

```
   ┌──────────────┐   ┌────────────┐   ┌──────────────┐
   │ Application │ ─>│  Outbox    │ ─>│  Publisher   │
   │             │   │  (table)   │   │  (NATS/etc)  │
   └──────────────┘   └────────────┘   └──────────────┘
```

1. App writes event + business data in same DB transaction (outbox row)
2. Background publisher reads outbox rows, dispatches to NATS
3. On success, marks outbox row as published
4. On failure, retries with exponential backoff

## Sagas

Long-running distributed transactions via compensating actions:

```rust
async fn provision_saga(req: SagaRequest) -> Result<()> {
    let saga = saga::builder()
        .step(create_sandbox).compensate(destroy_sandbox)
        .step(publish_artifact).compensate(unpublish)
        .build();
    saga.run(req).await
}
```

## Library layout

- `crates/eventbus/` - in-process broadcast
- `crates/saga/` - saga orchestrator
- `crates/outbox/` - durable outbox pattern
