# Session Overview

- Goal: provide a repo-aware `Taskfile.yml` with common `build`, `test`, `lint`, and `clean` tasks.
- Scope: `Taskfile.yml` at the repo root and the session notes under `docs/sessions/`.
- Success criteria: the Taskfile detects the repo's Go root, Rust driver, docs manifest, and Playwright subproject before running commands.
