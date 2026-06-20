# AGENTS.md — PhenoCompose

Phenotype repository

## Quick Links

- **Local CLAUDE.md:** See `CLAUDE.md` in this repository for project-specific guidance
- **Phenotype org governance:** `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`
- **Global agent guidance:** `~/.claude/AGENTS.md`
- **AgilePlus work tracking:** `cd /repos/AgilePlus && agileplus <command>`

## Key Workflows

1. **Before implementing:** Check AgilePlus for existing specs
2. **Quality gates:** Run linters, tests, and docs validation (see CLAUDE.md)
3. **Worktrees:** Use `repos/PhenoCompose-wtrees/<topic>/` for feature work
4. **Integration:** Commit to canonical repo (`main`) after quality gates pass

## Project-Specific Gotchas

See CLAUDE.md for language stack, build commands, and testing requirements.

## Architecture Decision Records

| ID | Title | Status | Location |
|----|-------|--------|----------|
| ADR-001 | Optimal Language Selection for NanoVMS | Proposed | [`docs/adr/ADR-001-language-selection.md`](docs/adr/ADR-001-language-selection.md) |
| ADR-002 | Three-Tier Isolation Architecture | --- | [`docs/adr/ADR-002-three-tier-isolation.md`](docs/adr/ADR-002-three-tier-isolation.md) |
| ADR-003 | WASM Runtime Selection | --- | [`docs/adr/ADR-003-wasm-runtime.md`](docs/adr/ADR-003-wasm-runtime.md) |
| ADR-004 | Storage Architecture | --- | [`docs/adr/ADR-004-storage-architecture.md`](docs/adr/ADR-004-storage-architecture.md) |
| ADR-005 | Networking Architecture | --- | [`docs/adr/ADR-005-networking-architecture.md`](docs/adr/ADR-005-networking-architecture.md) |
| ADR-006 | CLI Design | --- | [`docs/adr/ADR-006-cli-design.md`](docs/adr/ADR-006-cli-design.md) |
| ADR-007 | Testing Strategy | --- | [`docs/adr/ADR-007-testing-strategy.md`](docs/adr/ADR-007-testing-strategy.md) |
| ADR-008 | Deployment Strategy | --- | [`docs/adr/ADR-008-deployment-strategy.md`](docs/adr/ADR-008-deployment-strategy.md) |
| ADR-009 | Performance Targets | --- | [`docs/adr/ADR-009-performance-targets.md`](docs/adr/ADR-009-performance-targets.md) |
| ADR-010 | Security Model | --- | [`docs/adr/ADR-010-security-model.md`](docs/adr/ADR-010-security-model.md) |
| ADR-011 | Observability Stack | --- | [`docs/adr/ADR-011-observability-stack.md`](docs/adr/ADR-011-observability-stack.md) |
| ADR-012 | Multi-Tenancy Architecture | --- | [`docs/adr/ADR-012-multi-tenancy.md`](docs/adr/ADR-012-multi-tenancy.md) |
| ADR-013 | Game Automation Testing | --- | [`docs/adr/ADR-013-game-automation-testing.md`](docs/adr/ADR-013-game-automation-testing.md) |
| ADR-014 | Agent Desktop Environments | --- | [`docs/adr/ADR-014-agent-desktop-environments.md`](docs/adr/ADR-014-agent-desktop-environments.md) |

---

**Parent contract:** Extends Phenotype-org governance. See `CLAUDE.md` and parent `AGENTS.md` for complete operating procedures.
