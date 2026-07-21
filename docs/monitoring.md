# PhenoCompose Monitoring

PhenoCompose uses [OpenTelemetry](https://opentelemetry.io/) as the unified observability pipeline.

## Pipeline

```
   ┌─────────────┐      ┌─────────┐      ┌──────────────┐
   │ Application │ ───> │ OTel    │ ───> │ Grafana Cloud│
   │ (pheno-*    │      │ Collector│      │ + Prometheus│
   │  crates)    │      │ (OTLP)   │      │ + Tempo      │
   └─────────────┘      └─────────┘      └──────────────┘
```

- **Metrics**: Prometheus exporter, scraped every 15s
- **Traces**: OTLP to Tempo, sampled 10% in prod
- **Logs**: structured JSON to Loki via Promtail

## SLOs

| Service | SLO | Target | Error budget |
|---------|-----|--------|--------------|
| compose-build | p99 latency | < 500ms | 99.9% monthly |
| port-resolution | p99 latency | < 50ms | 99.95% monthly |
| publish | p99 latency | < 1s | 99.5% monthly |
| secret-store | p99 latency | < 100ms | 99.9% monthly |

## Dashboards

See `docs/dashboards/`:
- `red-method-rps.json` — Rate / Errors / Duration for the 4 port traits
- `use-resources.json` — CPU / memory / disk / network per adapter
- `golden-signals.json` — Latency / Traffic / Errors / Saturation
- `cost-attribution.json` — Per-tenant / per-adapter cost tracking
