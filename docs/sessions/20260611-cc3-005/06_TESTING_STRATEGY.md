# CC3-005 Testing Strategy

- Validate \`Taskfile.yml\` structure with \`task --list\` when the binary is available.
- Validate root package-script wiring with \`npm run test\`, \`npm run test:unit\`, and \`npm run test:coverage\` once dependencies are installed.
- Treat docs build plus unit-test invocation as the main regression path for this task.
