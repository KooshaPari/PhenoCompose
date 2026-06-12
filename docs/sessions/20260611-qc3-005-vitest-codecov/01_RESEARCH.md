# Research

- The repository already contained vitest.config.ts, but package.json had no Vitest scripts or dependencies.
- .github/workflows/ci.yml was the best integration point because it already owns Node install, build, and test behavior.
- codecov.yml was malformed YAML and mixed unrelated Go and Python assumptions, so a valid minimal Codecov config was required before upload integration.
- There is no tests tree yet, so coverage wiring must tolerate zero tests while still producing the standard Vitest coverage path once tests exist.
