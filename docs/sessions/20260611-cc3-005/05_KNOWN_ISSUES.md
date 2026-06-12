# CC3-005 Known Issues

- This worktree does not currently have installed root Node dependencies, so runtime verification of \`vitepress\` and \`vitest\` commands can fail until dependencies are installed.
- No TypeScript test files are currently present under the configured Vitest globs, so a green run depends on whether the intended behavior is "no tests" or future tests being added separately.
