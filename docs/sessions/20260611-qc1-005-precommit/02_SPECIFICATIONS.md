# Specifications

- Add hook coverage for \`prettier\`, \`eslint\`, \`tsc\`, \`vitest\`, \`markdownlint\`, and \`gitleaks\`.
- Keep the configuration repo-rooted and npm-script driven so contributors have one obvious install/run path.
- Avoid product-code changes; this task is tooling-only.

## ARUs

- Assumption: contributors will install Node dependencies before running the hooks.
- Risk: network-restricted execution prevents live dependency installation in this session.
- Uncertainty: some future markdown files may need additional lint rule tuning beyond disabling line-length.
