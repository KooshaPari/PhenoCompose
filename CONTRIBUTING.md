# Contributing to PhenoCompose

Thank you for your interest in PhenoCompose — the policy-as-code
composition engine at the heart of the Phenotype ecosystem. We welcome
bug reports, documentation improvements, tests, refactors, and new
adapters from everyone.

This document explains how to set up your development environment,
run the test suite, propose changes, and get them merged safely.

---

## 1. Code of Conduct

By participating, you agree to abide by the
[Phenotype Code of Conduct](CODE_OF_CONDUCT.md) (if present) and the
GitHub Community Guidelines. Be respectful, assume good faith, and
prefer written communication that can be quoted later.

## 2. Project Overview

PhenoCompose turns **policy fragments** — typed capability
declarations, resource limits, provenance tags — into executable
**compositions** that the rest of the Phenotype mesh can enforce.
The implementation is multi-language by design:

- A **Rust** core that owns the type system, the SAT-style constraint
  solver, and the cryptographic attestation pipeline.
- **Go** adapters (`adapters/`) that bridge the core to existing
  policy engines (OPA, Cedar, Rego).
- A **TypeScript** SDK (`bindings/ts/`) and a **Python** SDK
  (`bindings/py/`) for downstream application authors.
- A thin **C / C-ABI** shim (`bindings/c/`) for embedding in
  non-managed runtimes.

Each sub-tree has its own `Cargo.toml`, `go.mod`, `package.json`, or
`pyproject.toml`; the `Taskfile.yml` (and `justfile`) at the root
orchestrate them.

## 3. Development Environment

### 3.1 Required Toolchains

| Tool            | Version  | Why                            |
| --------------- | -------- | ------------------------------ |
| Rust            | `stable` | Core, type system, solver      |
| `cargo`         | ≥ 1.78   | Build, test, fmt, clippy       |
| `rustfmt`       | stable   | Formatting                     |
| `clippy`        | stable   | Lints (CI fails on warnings)   |
| `cargo-deny`    | ≥ 0.14   | License + advisory gating      |
| `cargo-audit`   | ≥ 0.20   | Vulnerability scan             |
| Go              | ≥ 1.22   | OPA / Cedar / Rego adapters    |
| `golangci-lint` | ≥ 1.55   | Aggregated Go lints            |
| Node.js         | ≥ 20 LTS | TypeScript SDK build           |
| `pnpm`          | ≥ 9      | TypeScript SDK package manager |
| Python          | ≥ 3.11   | Python SDK                     |
| `uv`            | ≥ 0.4    | Python env + dep manager       |
| `maturin`       | ≥ 1.7    | Build Python wheels from Rust  |
| `ruff`          | ≥ 0.5    | Python linter + formatter      |
| `mypy`          | ≥ 1.10   | Python type-check              |
| Task            | ≥ 3      | Cross-language task runner     |

### 3.2 Clone + Bootstrap

```bash
git clone https://github.com/KooshaPari/phenocompose.git
cd phenocompose
task bootstrap        # or: ./scripts/bootstrap.sh
```

`task bootstrap` will:

1. Install git hooks (`.githooks/` or `lefthook`).
2. Run `cargo fetch`, `go mod download`, `pnpm install`,
   `uv venv && uv pip install -e bindings/py`.
3. Run a smoke build of every language sub-project.
4. Run `cargo deny` and `cargo audit` to baseline the dependency
   tree.

### 3.3 Editor Setup

- **VS Code**: open `phenocompose.code-workspace` (if present);
  recommended extensions: `rust-lang.rust-analyzer`,
  `golang.go`, `tamasfe.even-better-toml`, `ms-python.python`,
  `charliermarsh.ruff`.
- **Neovim / Helix / Zed**: zero-config LSPs; the
  `rust-analyzer` config lives at `.config/rust-analyzer.toml`.
- **JetBrains RustRover + GoLand + PyCharm**: each sub-project
  opens independently; no meta-workspace file is shipped.

## 4. Building

```bash
# Everything (Rust + Go + TS + Python)
task build

# Just the Rust core
cargo build --workspace --all-targets

# Just the Go adapters
(cd adapters && go build ./...)

# TypeScript SDK
(cd bindings/ts && pnpm build)

# Python wheel (debug)
(cd bindings/py && uv run maturin develop --uv)
```

Useful binary outputs:

- `target/release/phenocompose` — main CLI.
- `target/release/phenocompose-verify` — standalone attestation
  verifier.
- `adapters/bin/phenocompose-opa` — OPA plugin.
- `bindings/py/target/wheels/phenocompose-*.whl` — Python wheel.

## 5. Testing

PhenoCompose has a tiered test pyramid:

| Tier        | Command                                                 | Owner     | Wall-clock |
| ----------- | ------------------------------------------------------- | --------- | ---------- |
| Unit (Rust) | `cargo test --workspace`                                | Core team | < 3 min    |
| Unit (Go)   | `(cd adapters && go test ./...)`                        | Adapter   | < 1 min    |
| Unit (TS)   | `(cd bindings/ts && pnpm test)`                         | TS SDK    | < 2 min    |
| Unit (Py)   | `(cd bindings/py && uv run pytest)`                     | Py SDK    | < 2 min    |
| Conformance | `task test:conformance`                                 | Core team | < 10 min   |
| Property    | `cargo test --features proptest`                        | Core team | < 5 min    |
| Fuzz        | `cargo +nightly fuzz run parser -- -max_total_time=600` | Security  | 10 min     |
| Cross-lang  | `task test:cross-language`                              | Core team | < 15 min   |

CI runs unit + conformance on every PR. Property, fuzz, and
cross-language run nightly and on release tags.

## 6. Coding Standards

- **Rust**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.
  No `unwrap()` in non-test code; use `anyhow::Context` or a typed
  error. Prefer `&str` over `String` in public APIs.
- **Go**: `gofmt -s`, `goimports -local github.com/KooshaPari/phenocompose`,
  `golangci-lint run`. Wrap errors with `%w`.
- **TypeScript**: `prettier --check .`, `eslint .`, `tsc --noEmit`.
  Discriminated unions over enums; `readonly` everywhere a value
  is logically immutable.
- **Python**: `ruff format`, `ruff check`, `mypy --strict`. Type
  hints are mandatory on all new code.
- **Public APIs**: every public function has a doc-comment. Enums
  exposed to multiple languages get a `#[non_exhaustive]` attribute.
- **Commits**: conventional commits — see §9.

## 7. Branching

- Default branch: `main`.
- Long-lived integration branches: `release/X.Y`.
- Feature / fix / chore branches: `<type>/<scope>-<short-desc>`
  (kebab-case, ≤ 60 chars). The `<type>` matches the conventional
  commit type and the `<scope>` matches the commit scope.
  Examples: `feat/solver-partial-assign`,
  `fix/opa-adapter-context-leak`, `chore/l2-30-governance-2026-06-11`.

## 8. Pull Request Process

1. **Open an issue first** for non-trivial changes. Bug fixes and
   documentation improvements may go straight to PR.
2. **Fork** the repo (or push to a feature branch if you have write
   access via the Phenotype org).
3. **Keep PRs focused**: < 400 lines diff where possible. Split
   larger refactors into a stack of dependent PRs.
4. **Fill the PR template** — it links to the design doc / spec /
   issue, the test plan, and the rollout / risk notes.
5. **Pass CI**: fmt, clippy, all tier-1 tests, `cargo deny` (license +
   advisory), `cargo audit`, CodeQL, OpenSSF Scorecard check.
6. **Request a review** from the CODEOWNERS — for PhenoCompose the
   default reviewer is `@KooshaPari`. Add a domain reviewer (e.g.
   security, SDK) for cross-cutting changes.
7. **Address review feedback** in additional commits; the maintainer
   will squash-merge once the conversation is resolved.
8. **After merge**, delete the source branch.

## 9. Commit Message Format (Conventional Commits)

PhenoCompose uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

```
<type>(<scope>): <short summary>

<body — wrap at 72 cols; explain *what* and *why*>

<footer — e.g. "BREAKING CHANGE: ...", "Closes #123", "Refs: SPEC-42">
```

### Allowed types

| Type       | Semantics                                                |
| ---------- | -------------------------------------------------------- |
| `feat`     | A new user-facing feature                                |
| `fix`      | A bug fix                                                |
| `docs`     | Documentation only                                       |
| `style`    | Whitespace/formatting, no code change                    |
| `refactor` | Code change that neither fixes a bug nor adds a feature  |
| `perf`     | Performance improvement                                  |
| `test`     | Add or correct tests                                     |
| `build`    | Build system, CI, or dependency change                   |
| `chore`    | Tooling, repo hygiene, governance (this PR)              |
| `revert`   | Reverts a previous commit (include `Reverts: <sha>`)     |
| `security` | Security fix (also notify `security@phenotype.internal`) |

### Scopes (non-exhaustive)

`core`, `solver`, `types`, `crypto`, `adapters/opa`,
`adapters/cedar`, `bindings/ts`, `bindings/py`, `bindings/c`,
`cli`, `ci`, `docs`, `deps`, `governance`.

### Examples

```
feat(solver): support partial assignments for unsat cores

The SAT-style solver used to refuse partial assignments on
unsatisfiable cores. We now return a per-variable domain
narrowing so the caller can decide whether to retry with
relaxed constraints. The new behaviour is opt-in via the
`allow_partial` flag on `Solver::solve`.

Adds a conformance case under `tests/conformance/partial/`.

Refs: SPEC-04 §6
```

```
fix(adapters/opa): close rego context on adapter shutdown

The OPA adapter leaked one `rego.EvalContext` per reconfigure
cycle because the previous handle was overwritten before
`Close()` was called. We now call `Close()` from the
`Drop` impl, gated on a `closed` flag to make it idempotent.

Fixes #1307
```

## 10. Reviewer Expectations

- **First response** within 2 business days.
- Reviews cover: correctness, test coverage, security, performance,
  API stability, observability, and documentation.
- Maintainer privilege: squash-merge with the PR title as the squash
  subject and the PR body as the squash body. Override only when the
  history itself is meaningful (rare; discuss in the PR).

## 11. Release Process

PhenoCompose follows semver. Releases are cut from `main` by the
release-please GitHub App configured in
`.github/release-please-config.json`. The maintainer approves the
release PR, which is auto-generated and bumps versions, CHANGELOG,
and tags. Cross-language artifacts (Rust crates, npm packages,
PyPI wheels, OCI images) are published in a single coordinated
release.

## 12. Getting Help

- **Discord**: `#phenocompose` on the Phenotype Discord.
- **Discussions**: GitHub Discussions → _Q&A_.
- **Office hours**: Mondays 16:00 UTC, calendar link in the
  pinned issue.

Welcome aboard — we are glad you are here.
