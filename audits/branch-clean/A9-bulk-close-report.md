# A9: Bulk-close 2026-06-11 CC/QC/SD SOTA Chore Branches

- **Date:** 2026-07-02 (execution) / 2026-06-25 (initial inventory)
- **DAG Unit:** A9
- **Type:** branch-clean
- **Epic:** epic_A -- Hygiene garden & branch slim
- **Repository:** PhenoCompose (`KooshaPari/PhenoCompose`)
- **Executor:** compose-pool/A9 subagent (DAG phase 1)
- **Branch:** `branch-clean/A9-bulk-close-cc-qc-sd` (off `main@43abf2f`)
- **Standards:** phenotype-registry (DOMAIN_ROLES, SOTA, LANGUAGE_PLACEMENT); agileplus 9-phase pipeline
- **Workflow step reached:** Dispatch (local) + audit-record; Merge deferred (no remote push per subagent scope)

## Executive Summary

All 11 target SOTA chore branches exist as **remote-only** tracking refs.
None have been merged into current `origin/main` (which was force-reset
to `43abf2f` after the initial audit on 2026-06-22 against `64cc0491`).
None of their tip commits are ancestors of current main. All are stale
(20 days since last commit) and safe for remote deletion.

The 11 branch tips have been **pre-archived** as lightweight tags under
`refs/tags/archive/<X>-2026-06-11` with identical SHAs, so the work is
fully reversible after the bulk-close.

## Branch Inventory (pre-deletion)

### CC Series (Conformance / Compliance Cleanup)

| Branch | Last Commit | Date | Ahead/Behind main | Archive tag | Match |
|--------|-----------|------|-------------------|-------------|-------|
| `chore/CC1-005-sota-2026-06-11` | dc40a11 | 2026-06-12 | 91/13 | `archive/CC1-2026-06-11` | YES |
| `chore/CC2-005-sota-2026-06-11` | 0b72602 | 2026-06-12 | 91/13 | `archive/CC2-2026-06-11` | YES |
| `chore/CC3-005-sota-2026-06-11` | 08f93ed | 2026-06-14 | 93/13 | `archive/CC3-2026-06-11` | YES |
| `chore/CC4-005-sota-2026-06-11` | 5ae4e2c | 2026-06-12 | 91/13 | `archive/CC4-2026-06-11` | YES |

### QC Series (Quality / Code-quality Cleanup)

| Branch | Last Commit | Date | Ahead/Behind main | Archive tag | Match |
|--------|-----------|------|-------------------|-------------|-------|
| `chore/QC1-005-sota-2026-06-11` | c24feaa | 2026-06-12 | 91/13 | `archive/QC1-2026-06-11` | YES |
| `chore/QC2-005-sota-2026-06-11` | 8054b37 | 2026-06-12 | 91/13 | `archive/QC2-2026-06-11` | YES |
| `chore/QC3-005-sota-2026-06-11` | da529d9 | 2026-06-12 | 91/13 | `archive/QC3-2026-06-11` | YES |
| `chore/QC4-005-sota-2026-06-11` | 83c546c | 2026-06-12 | 91/13 | `archive/QC4-2026-06-11` | YES |

### SD Series (Standards / SOTA Documentation)

| Branch | Last Commit | Date | Ahead/Behind main | Archive tag | Match |
|--------|-----------|------|-------------------|-------------|-------|
| `chore/SD2-004-sota-2026-06-11` | d882d3c | 2026-06-12 | 91/13 | `archive/SD2-2026-06-11` | YES |
| `chore/SD3-004-sota-2026-06-11` | 4638a5a | 2026-06-12 | 91/13 | `archive/SD3-2026-06-11` | YES |
| `chore/SD4-004-sota-2026-06-11` | 53d84ee | 2026-06-12 | 91/13 | `archive/SD4-2026-06-11` | YES |

## Summary Metrics

| Metric | Value |
|--------|-------|
| Branches assessed | 11 |
| Local branches found | 0 |
| Remote-only branches | 11 |
| Merged into current main (`43abf2f`) | 0 |
| Safe to delete remote | 11 |
| Archive tags present and matching | 11 |
| Local remote-tracking refs deleted by this unit | 11 |
| Remote refs deleted from origin (push) | **0** (deferred -- see "Open actions") |

## Reversibility

Each deleted branch can be restored from its archive tag with:

```bash
git branch chore/<NAME>-sota-2026-06-11 archive/<X>-2026-06-11
git push -u origin chore/<NAME>-sota-2026-06-11
```

where `<NAME>` is `CC1`/`CC2`/.../`SD4` and `<X>` matches.

## Action Plan (Recorded for Human Operator)

The remote push was **not executed** by this subagent per the parent
task scope ("Do NOT push to remote unless the file explicitly says to"
-- the DAG template references a generic "Push." step but the subagent
constraint overrides it; also no git credentials available in env).

The exact one-shot command is preserved at:

- `audits/branch-clean/evidence/A9-remote-delete.sh`

Evidence files captured during this run:

- `audits/branch-clean/evidence/A9-pre-remote-refs.txt` -- full pre-state table
- `audits/branch-clean/evidence/A9-pre-tip-shas.txt` -- 11 tip SHAs
- `audits/branch-clean/evidence/A9-pre-tip-details.txt` -- 11 tip date+subject
- `audits/branch-clean/evidence/A9-archive-tag-shas.txt` -- archive tag SHAs (verify match)
- `audits/branch-clean/evidence/A9-archive-tags.txt` -- archive tag subject+date
- `audits/branch-clean/evidence/A9-remote-delete.sh` -- bash script for human operator
- `audits/branch-clean/evidence/A9-post-archive-tags-still-exist.txt` -- proof tags survived deletion

## Diffstat (this branch vs main)

```
.git/refs/remotes/origin/chore/CC1-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/CC2-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/CC3-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/CC4-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/QC1-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/QC2-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/QC3-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/QC4-005-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/SD2-004-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/SD3-004-sota-2026-06-11  | 1 -
.git/refs/remotes/origin/chore/SD4-004-sota-2026-06-11  | 1 -
audits/branch-clean/A9-bulk-close-report.md             | modified (this file)
audits/branch-clean/evidence/A9-*                       | 7 new files
```

11 files changed (refs), 0 insertions(+), 11 deletions(-) on the git
ref DB; plus documentation additions under `audits/branch-clean/`.

## Open Actions

1. **Human operator**: review and execute `audits/branch-clean/evidence/A9-remote-delete.sh`
   against `origin` to remove the remote refs.
2. **Validator subagent** (per DAG contract): re-run `grade.sh --json`
   on `branch-clean/A9-bulk-close-cc-qc-sd` and diff vs `main@43abf2f`.
3. **Auditor loop**: if grade regresses, auto-add units to
   `phenotype-registry/audits/auto_added.yaml`.

## Constraints Honored

- [x] UTF-8 ASCII only (no smart quotes, em-dashes)
- [x] No secrets committed
- [x] No history rewrite of shared branches
- [x] Did not touch `phenotype-sdk` (not in scope)
- [x] DOMAIN_ROLES respected -- bulk-close audit lives in PhenoCompose (the owner repo for the branches)
- [x] No dogfood fixes required -- `git branch -dr` and `git rev-parse` worked as documented
- [x] No PR opened (no push; per subagent scope)
- [x] Audit evidence preserved under `audits/branch-clean/evidence/`