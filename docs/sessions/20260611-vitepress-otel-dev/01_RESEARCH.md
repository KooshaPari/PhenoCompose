# Research

## In-repo findings

- package.json exposes VitePress only through docs:dev, docs:build, and docs:preview.
- docs/.vitepress/config.mts is pure site config and does not control the Node process that launches the dev server.
- No existing Node-side telemetry bootstrap exists in the docs toolchain.

## Decision

- Instrument the Node process before VitePress CLI startup with node --import.
- Keep tracing scoped to the dev server path rather than all npm scripts.
- Use OpenTelemetry Node SDK plus OTLP HTTP exporter so trace delivery is configured entirely by standard environment variables.
