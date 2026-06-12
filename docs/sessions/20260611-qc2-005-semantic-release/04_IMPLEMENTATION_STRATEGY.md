# Implementation Strategy

- Keep the release change isolated to the docs package and GitHub workflow surface.
- Use semantic-release as the single release orchestrator to avoid maintaining parallel release metadata paths.
- Reuse the repository's existing Node 20 workflow baseline.
- Avoid lockfile churn because the repository does not currently track a JavaScript lockfile.
