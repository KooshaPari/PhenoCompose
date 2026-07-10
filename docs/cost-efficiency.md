# PhenoCompose Cost Efficiency

Per-adapter + per-tenant cost tracking.

## Cost dimensions

| Dimension | Metric | Unit |
|-----------|--------|------|
| Compute | vCPU-second | $/vCPU-s |
| Memory | GB-second | $/GB-s |
| Storage | GB-month | $/GB-month |
| Network egress | GB | $/GB |

## Cost attribution

```rust
pub struct CostReport {
    pub sandbox_id: String,
    pub vcpu_seconds: f64,
    pub memory_gb_seconds: f64,
    pub storage_gb: f64,
    pub network_egress_gb: f64,
    pub estimated_usd: f64,
}

pub trait CostTracker {
    fn record(&self, event: CostEvent) -> Result<(), String>;
    fn report(&self, period: Period) -> CostReport;
}
```

## Real-time tracking

```rust
let tracker = CostTracker::postgres("postgres://...").await?;
tracker.record(CostEvent::SandboxStarted { id, vcpu: 2, mem_gb: 1.0 }).await?;
tracker.record(CostEvent::NetworkEgress { bytes: 1024 }).await?;
```

## Per-tenant billing

```rust
tracker.report(Period::Month(2026, 7)).by_tenant("user-123")
// CostReport { vcpu_seconds: 86400, memory_gb_seconds: 43200, estimated_usd: 12.34 }
```

## Budget enforcement

```rust
tracker.set_budget("user-123", 100.00).await?;
match tracker.try_acquire("user-123", CostEvent::SandboxStarted { ... }).await {
    Ok(()) => { /* proceed */ }
    Err(BudgetExceeded) => { /* reject */ }
}
```

## Optimization targets

- **P50 cost per sandbox**: < $0.10/hour
- **P99 cost per sandbox**: < $1.00/hour
- **Idle cost** (stopped but not reclaimed): < $0.01/hour
- **Cost per GB-second**: < $0.0001

## CI integration

- Cost report on every PR
- Alert on >20% cost regression
- Weekly cost reports emailed to admins
