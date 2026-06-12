# Session Overview

- Goal: add OpenTelemetry tracing to the VitePress dev server startup path with minimal repo churn.
- Scope: VitePress startup script wiring, preload bootstrap, and focused validation notes.
- Success criteria: npm run docs:dev boots VitePress through a Node preload that starts an OpenTelemetry SDK and exports traces through standard OTLP env vars.
