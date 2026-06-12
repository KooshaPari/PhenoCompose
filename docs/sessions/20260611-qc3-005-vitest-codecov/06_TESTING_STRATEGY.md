# Testing Strategy

- Validate package.json and workflow and YAML syntax locally.
- Confirm the updated Vitest config includes lcov output and passWithNoTests.
- Defer full npm install and vitest execution because the environment has no local dependencies and network access is restricted.
