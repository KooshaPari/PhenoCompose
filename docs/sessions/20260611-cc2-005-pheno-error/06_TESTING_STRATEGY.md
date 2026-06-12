# Testing Strategy

- Use `vitest` for unit tests inside `bindings/ts`.
- Validate:
  - successful sync wrapping
  - failed sync wrapping
  - successful async wrapping
  - failed async wrapping
  - normalization of `Error`, string, and object throwables
- Verify with:
  - `npm --prefix bindings/ts run build`
  - `npm --prefix bindings/ts run test -- --run`

