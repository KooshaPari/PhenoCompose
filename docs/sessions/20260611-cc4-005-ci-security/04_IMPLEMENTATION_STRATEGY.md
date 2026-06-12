# Implementation Strategy

- Extend the existing \`ci.yml\` instead of creating another workflow so dependency scanning remains part of the main CI surface.
- Keep the existing \`test\` job unchanged.
- Add a separate \`dependency_audit\` job so scan failures are isolated and easy to read.

