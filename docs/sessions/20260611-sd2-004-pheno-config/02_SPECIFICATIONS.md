# Specifications

- Create packages/pheno-config as the reusable package boundary.
- Export typed docs configuration primitives plus the shared docs site config object.
- Update docs/.vitepress/config.mts to import shared config rather than defining inline site structure.

## ARUs

- Assumption: cross-repo consumers can import ESM .mts package sources directly or through a later packaging step.
- Risk: local dependency installation is unavailable in this environment, so verification is limited to structural and syntax-level checks.
- Uncertainty: downstream repos that will consume this package are not present in this worktree.
