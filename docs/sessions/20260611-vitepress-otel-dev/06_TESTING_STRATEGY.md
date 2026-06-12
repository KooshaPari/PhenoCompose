# Testing Strategy

- Validate JSON syntax in package.json.
- Validate preload module syntax with node --check docs/.vitepress/opentelemetry.mjs.
- When dependencies are installed, run npm run docs:dev with OTLP env vars pointed at a collector and confirm spans arrive.
- Because node_modules is absent in this worktree, dependency installation and live dev-server verification are deferred.
