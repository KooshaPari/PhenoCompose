# Research

- Existing docs config lived entirely in docs/.vitepress/config.mts.
- No existing packages/, tsconfig.json, or reusable TypeScript config package existed in the repo.
- Root package.json only exposed VitePress scripts, so the extraction needed to preserve current docs build behavior.
