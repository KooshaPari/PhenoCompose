# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial hygiene bootstrap: SECURITY.md, CHANGELOG.md, CONTRIBUTING.md, CODEOWNERS.
- `justfile` at the repo root as the canonical task runner (replaces `Taskfile.yml`). Recipes: `default` (lists recipes), `list`, `build`, `test`, `lint`, `clean`. Install `just` from https://just.systems to use it.

### Changed

- Migrated task definitions from `Taskfile.yml` (go-task) to `justfile` (casey/just). Recipes preserve the original target names and behavior: Go/Rust/JS detection, JS runner prefers `bun` then falls back to `npm`, `golangci-lint` falls back to `go vet`.

### Deprecated

### Removed

- `Taskfile.yml` — superseded by `justfile`.

### Fixed

### Security
