# CC3-005 Implementation Strategy

- Use \`package.json\` as the single source of truth for Node commands.
- Keep \`Taskfile.yml\` thin so task execution stays aligned with package scripts.
- Let \`test\` cover the higher-level documentation-plus-unit-test path, while \`test-unit\` and \`test-coverage\` remain focused Vitest entrypoints.
