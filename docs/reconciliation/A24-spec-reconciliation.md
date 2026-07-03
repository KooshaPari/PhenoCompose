# A24 — SPEC.md Reconciliation Report

**Date**: 2026-06-25
**Repo**: PhenoCompose
**Analyst**: DAG unit A24
**Branch**: docs/A24-reconcile-spec-md

---

## 1. Findings Summary

| Item | Status |
|------|--------|
| `docs/SPEC.md` exists? | **NO** — SPEC.md is at repo root, not under `docs/` |
| `SPEC.md` in `origin/main`? | YES — full spec with Stack, Commands, Design Decisions |
| `docs/operations/iconography/SPEC.md` | YES — unrelated iconography standard (5 lines) |
| Architecture doc drift? | **None** — `docs/reference/architecture.md` identical across all branches |

## 2. Branch analysis

### `origin/chore/l5-83-phenocompose-integration-2026-06-11`

- Key commit: `0b2a210 docs(focus-repos): add SPEC.md to all focus repos (L5 #87)`
- Adds full SPEC.md with Stack table, Key Commands, Design Decisions, Integration Points
- Adds `packages/` npm workspace (integration, pheno-config, pheno-errors, pheno-tracing)
- Adds Rust `pheno-compose-driver/` crate
- Adds `ports/` Rust crate
- Adds `.grade-reports/` infrastructure

### `origin/chore/l5-87-spec-arch-2026-06-11`

- Strips operational sections from SPEC.md:
  - Stack table **removed**
  - Key Commands table **removed**
  - Design Decisions **removed**
  - Integration Points **removed**
- Removes entire `packages/` npm workspace
- Removes CI additions from justfile
- Adds `worklog-L2-029-2026-06-11.json`
- **Intent**: Spec-arch cleanup, slimming SPEC.md to technical survey content only

### Diff: l5-83 vs l5-87 (17 files differ)

```
.github/dependabot.yml    | +/- 192 lines
SPEC.md                   | 33 lines removed (4 sections deleted)
justfile                  | 17 lines removed
packages/ (6 files)       | Entire npm workspace removed
worklog-L2-029-*.json     | 55 lines added
```

## 3. Drift Assessment

| SPEC.md Section | main | l5-83 | l5-87 |
|-----------------|------|-------|-------|
| Overview | YES | YES | YES (identical) |
| Stack table | YES | YES | **DELETED** |
| Key Commands | YES | YES | **DELETED** |
| Design Decisions | YES | YES | **DELETED** |
| Integration Points | YES | YES | **DELETED** |
| Part I SOTA Landscape | YES | YES | YES (format diff only) |
| Parts II-VI, Appendixes | YES | YES | YES (identical) |

Architecture docs: `docs/reference/architecture.md` — **no drift** across all branches.

## 4. Recommendations

1. **SPEC.md location**: File is at root `SPEC.md`, not `docs/SPEC.md`. If `docs/SPEC.md` is the canonical path, symlink or migrate.
2. **l5-87 stripping**: The operational metadata removal is intentional cleanup. Merge into main after verifying content is preserved in ADRs/reference docs.
3. **Packages divergence**: The `packages/` npm workspace is in l5-83 but absent from l5-87. Must reconcile if integration packages are needed.
4. **Neither branch merged**: Both l5-83 and l5-87 are unmerged into main — drift exists only in feature branches.

## 5. SPEC-related files in docs/

- `docs/operations/iconography/SPEC.md` — Iconography standard (unrelated, 5 lines)
- `docs/sessions/20260428-taskfile-pheno-compose/02_SPECIFICATIONS.md` — Session spec notes
- `docs/reference/architecture.md` — Architecture reference (283 lines, stable)

## 6. Environment

- Working tree: `docs/A24-reconcile-spec-md` (forked from `origin/main`)
- Grade infra: `.grade-reports/grade.json` present on l5-83/l5-87, absent from main
