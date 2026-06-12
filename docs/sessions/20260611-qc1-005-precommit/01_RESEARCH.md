# Research

- Existing state: root \`.pre-commit-config.yaml\` already covered hygiene hooks, \`gitleaks\`, and a manual \`tsc\` hook, but it did not define \`prettier\`, \`eslint\`, \`vitest\`, or \`markdownlint\`.
- Existing state: root \`package.json\` only contained VitePress docs scripts and lacked the toolchain dependencies needed to support the requested hooks.
- Existing state: \`vitest.config.ts\`, \`playwright.config.ts\`, and \`docs/.vitepress/config.mts\` already existed, but there was no root \`tsconfig.json\` or ESLint/Prettier/Markdownlint configuration.
- Decision: add a small root TypeScript lint/format scaffold so the hook entries point at concrete scripts and config instead of dead commands.
