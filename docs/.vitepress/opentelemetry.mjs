import process from 'node:process'

import { diag, DiagConsoleLogger, DiagLogLevel } from '@opentelemetry/api'
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node'
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http'
import { resourceFromAttributes } from '@opentelemetry/resources'
import { NodeSDK } from '@opentelemetry/sdk-node'
import { ATTR_SERVICE_NAME, ATTR_SERVICE_VERSION } from '@opentelemetry/semantic-conventions'

const logLevel = process.env.OTEL_LOG_LEVEL?.toLowerCase()
const tracesEndpoint =
  process.env.OTEL_EXPORTER_OTLP_TRACES_ENDPOINT ??
  process.env.OTEL_EXPORTER_OTLP_ENDPOINT

if (logLevel === 'debug') {
  diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.DEBUG)
}

if (process.env.OTEL_SDK_DISABLED === 'true' || !tracesEndpoint) {
  if (logLevel === 'debug') {
    const reason =
      process.env.OTEL_SDK_DISABLED === 'true'
        ? 'OTEL_SDK_DISABLED=true'
        : 'missing OTEL_EXPORTER_OTLP_ENDPOINT'
    process.stderr.write(`[otel] vitepress dev tracing disabled (${reason})\n`)
  }
} else {
  const serviceName = process.env.OTEL_SERVICE_NAME ?? 'nanovms-docs-vitepress-dev'
  const serviceVersion = process.env.npm_package_version ?? '0.1.0'

  const sdk = new NodeSDK({
    resource: resourceFromAttributes({
      [ATTR_SERVICE_NAME]: serviceName,
      [ATTR_SERVICE_VERSION]: serviceVersion,
    }),
    traceExporter: new OTLPTraceExporter(),
    instrumentations: [
      getNodeAutoInstrumentations({
        '@opentelemetry/instrumentation-fs': {
          enabled: false,
        },
      }),
    ],
  })

  await sdk.start()

  const shutdown = async (signal) => {
    try {
      await sdk.shutdown()
      if (logLevel === 'debug') {
        process.stderr.write(`[otel] vitepress dev tracing shutdown on ${signal}\n`)
      }
    } catch (error) {
      console.error('OpenTelemetry shutdown failed', error)
    } finally {
      process.exit(0)
    }
  }

  process.once('SIGINT', () => {
    void shutdown('SIGINT')
  })
  process.once('SIGTERM', () => {
    void shutdown('SIGTERM')
  })
}
