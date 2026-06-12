# AGENTS.md — docs

This file governs `/docs` and all child paths beneath it.

## Purpose

- Treat `docs/` as the VitePress documentation root.
- Keep documentation edits aligned with the site structure in `docs/.vitepress/`.
- Prefer updating existing indexes and section pages instead of creating redundant standalone notes.

## Agent Discovery

Use these commands from the repository root when orienting inside the docs tree:

```bash
rg --files docs
rg -n "vitepress|sidebar|nav|search" docs/.vitepress package.json justfile
sed -n '1,220p' docs/.vitepress/config.mts
sed -n '1,220p' docs/README.md
```

- `docs/.vitepress/config.mts` is the authoritative source for navigation, sidebar, and search behavior.
- `docs/README.md` is the human index for top-level documentation sections.
- If you add a new top-level docs section, update `docs/README.md` in the same change.

## VitePress Commands

Use the package scripts declared in `package.json`:

```bash
npm run docs:dev
npm run docs:build
npm run docs:preview
```

- For validation after docs edits, run `npm run docs:build`.
- The repo `just build` and `just lint` flows also execute the docs build when docs scripts are present.

## Editing Rules

- Keep Markdown paths and links relative to the current docs location unless the file clearly requires absolute repo paths.
- Reuse existing section conventions in `adr/`, `guide/`, `guides/`, `journeys/`, `reference/`, `stories/`, and `traceability/`.
- Do not add temporary docs filenames such as `*_final.md`, `*_v2.md`, or `*_draft.md`.
