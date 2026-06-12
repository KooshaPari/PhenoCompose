# Known Issues

- Node dependencies were not installed in-session because network access is restricted here, so runtime verification of \`npm exec\`, \`eslint\`, \`prettier\`, \`tsc\`, \`vitest\`, and \`markdownlint-cli2\` is deferred until the repo dependencies are installed locally or in CI.
- The hook commands are configured to tolerate an empty Vitest test set via \`--passWithNoTests\`, which matches the current sparse TS test surface.
