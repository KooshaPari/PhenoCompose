# Implementation Strategy

- Keep the package single-purpose and small: one source module and one test module.
- Prefer pure data errors over subclass-heavy exception hierarchies so the API stays serializable and cross-runtime friendly.
- Re-export `neverthrow` primitives to make the package the canonical entrypoint for future TS code in this repo.
- Wire root `npm` scripts instead of changing CI semantics broadly.

