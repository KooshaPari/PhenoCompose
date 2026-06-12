# Implementation Strategy

- Keep the tracing change isolated to the docs dev-server path instead of broadening it to build or preview commands.
- Start the OpenTelemetry Node SDK from a preload so HTTP and other Node instrumentation can attach before VitePress initializes.
- Leave tracing dormant unless an OTLP endpoint is explicitly configured, which preserves the default local dev experience when no collector is running.
