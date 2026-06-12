# Implementation Strategy

- Use local \`pre-commit\` hooks for JS/TS tools so the repo delegates execution to installed project dependencies.
- Add lightweight root config files only where the hook commands require them: ESLint flat config, TypeScript config, Prettier config, Markdownlint config.
- Preserve the existing \`gitleaks\` workflow and generic hygiene hooks instead of replacing them with bespoke equivalents.
