# Implementation Strategy

- Keep the Taskfile at the repository root.
- Use shell gating on `go.mod`, `pheno-compose-driver/Cargo.toml`, `package.json`, and `tests/playwright/package.json`.
- Prefer the repo's existing commands instead of inventing new wrappers.
- Keep the task surface limited to `build`, `test`, `lint`, and `clean` so the file stays predictable and easy to invoke.
