# CLAUDE.md — PhenoCompose

Extends parent governance. See:

- Global baseline: `~/.claude/CLAUDE.md`
- Phenotype root: `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`
- AgilePlus mandate: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- Governance reference: `AGENTS.md` (local, this repository)

## Project Overview

**Name:** PhenoCompose
**Description:** Unified Docker Compose evolution + KVMS integration. Driver layer for nanovms. **NOTE: This repo shares the same Go module (`github.com/kooshapari/nanovms`) as the nanovms repo. Consider consolidating.
**Language Stack:** Go, TypeScript, Python
**Location:** `repos/PhenoCompose`
**Status:** Active

## AgilePlus Mandate

All work MUST be tracked in AgilePlus:

- CLI: `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`
- Check for existing specs before implementing
- Create spec for new work: `agileplus specify --title "<feature>" --description "<desc>"`
- No code without corresponding AgilePlus spec

## Architecture

PhenoCompose is the unified interface layer that combines:

- Docker Compose evolution (container orchestration)
- KVMS driver (Firecracker-based microVMs)
- Cross-platform support (macOS/Linux/Windows)

```
┌─────────────────────────────────────────────────┐
│  PhenoCompose CLI (Rust/Go hybrid)             │
├─────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ KVMS Driver │  │ Docker Compose Adapter  │  │
│  │  (Firecracker)│ │  (containerized)     │  │
│  └─────────────┘  └─────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Shared runtime: github.com/kooshapari/nanovms  │
└─────────────────────────────────────────────────┘
```

## Quality Checks

From this repository root:

```bash
# Build
go build ./...

# Test
go test ./...

# Lint
go fmt ./... && go vet ./...
```

## Worktree & Git Discipline

- Feature work uses repo-specific worktrees: `repos/[PROJECT]-wtrees/<topic>/`
- Canonical repo stays on `main` except during explicit merge operations
- All feature branches are temporary; integrate via pull request or squash commit

## Related Documents

- `README.md` — Project overview and quick start
- `PLAN.md` — Implementation plan
- `SPEC.md` — Specification
- `AGENTS.md` — AI agent instructions
- `CHANGELOG.md` — Version history

## Cross-Project Reuse

PhenoCompose shares its Go module with `nanovms` (same `github.com/kooshapari/nanovms`). For any shared runtime code, use the nanovms repo as the canonical home.

---

For CI, scripting language hierarchy, and other policies, see the canonical sources listed above.
