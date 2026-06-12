# Specifications

## Public API

- Export `Result`, `ResultAsync`, `ok`, and `err` from `neverthrow`.
- Export a typed `PhenoError` object shape with stable fields:
  - `code`
  - `message`
  - `details?`
  - `cause?`
- Export helpers:
  - `createPhenoError(...)`
  - `toPhenoError(...)`
  - `fromThrowable(...)`
  - `fromPromise(...)`

## Acceptance Criteria

- Sync code can be wrapped into `Result<T, PhenoError>`.
- Async code can be wrapped into `ResultAsync<T, PhenoError>`.
- Thrown strings, `Error` instances, and arbitrary objects all normalize to `PhenoError`.
- TypeScript build and tests pass with the new package.

## ARUs

- Assumption: `CC2-005` is best satisfied by creating the missing TypeScript SDK surface referenced by repo docs.
- Risk: CI uses `npm` while docs mention `pnpm`; root scripts are adjusted to keep the task verifiable without broader tooling migration.
- Uncertainty: there is no existing canonical npm scope in this repo, so package naming is kept repo-local and descriptive.

