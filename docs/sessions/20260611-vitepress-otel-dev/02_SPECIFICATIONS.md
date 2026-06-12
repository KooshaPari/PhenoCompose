# Specifications

- Add a Node preload at \`docs/.vitepress/opentelemetry.mjs\`.
- Route \`npm run docs:dev\` through \`node --import\` so tracing starts before the VitePress CLI boots.
- Use standard OpenTelemetry environment variables for export configuration:
  - \`OTEL_EXPORTER_OTLP_TRACES_ENDPOINT\`
  - fallback \`OTEL_EXPORTER_OTLP_ENDPOINT\`
- Default service identity:
  - service name: \`nanovms-docs-vitepress-dev\`
  - service version: \`package.json\` version
