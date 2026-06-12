# Implementation Strategy

- Keep the change narrow and additive by using the existing Vitest config instead of introducing a parallel test stack.
- Standardize on coverage/lcov.info, which is the common Codecov input path for Vitest.
- Use the existing ci.yml Node workflow for upload so Codecov is tied to the main JavaScript test execution path.
- Preserve green CI in the current repo state by enabling passWithNoTests until real Vitest suites land.
