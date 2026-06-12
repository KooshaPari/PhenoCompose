# Research

## In-Repo Findings

- `CONTRIBUTING.md` already documents a TypeScript SDK at `bindings/ts/`, including `pnpm build` and `pnpm test`, but the directory is absent in the current worktree.
- Root CI currently runs `npm install`, `npm run build`, and `npm test` from the repository root, so the TypeScript package must be reachable from root scripts.
- Existing top-level TypeScript is limited to config files (`vitest.config.ts`, `playwright.config.ts`, `docs/.vitepress/config.mts`), so there is no prior application-layer Result pattern to reuse.

## Decision

- Implement `bindings/ts` as the missing TypeScript SDK surface and make `pheno-error` its initial exported module.
- Use `neverthrow` directly rather than inventing a custom Result monad.
- Keep the first package intentionally small: typed error model, normalization helpers, promise/function wrappers, and tests.

