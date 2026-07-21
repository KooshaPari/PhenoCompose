# Release Process

`release-plz` automates version bumps, changelog generation, and GitHub releases.

## Workflow

1. CI runs `release-plz release --dry-run` on every PR (preview)
2. On merge to main, `release-plz release` runs:
   - Bumps versions in `Cargo.toml` (semver)
   - Updates `CHANGELOG.md` per package (conventional commits)
   - Creates GitHub release with notes
3. Tag triggers `cargo dist` for binaries (future)

## Conventional Commits

- `feat:` minor bump
- `fix:` patch bump
- `feat!:` or `BREAKING CHANGE:` major bump
- `chore:`, `docs:`, `refactor:`, `test:`, `ci:` no bump

See `.github/workflows/release.yml` for the workflow file.
