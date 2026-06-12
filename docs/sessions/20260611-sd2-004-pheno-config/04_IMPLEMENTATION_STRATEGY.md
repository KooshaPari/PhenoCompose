# Implementation Strategy

- Keep VitePress as the runtime owner of defineConfig.
- Move reusable content and types into packages/pheno-config/src/docs.mts.
- Use root workspaces metadata so the extracted package has a first-class home without changing existing docs scripts.
