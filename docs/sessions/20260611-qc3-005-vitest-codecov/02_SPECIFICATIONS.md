# Specifications

- Add test and test:coverage npm scripts backed by Vitest.
- Ensure Vitest emits lcov output under coverage.
- Update CI to run coverage and upload coverage/lcov.info with Codecov.
- Add a Codecov badge to the root README.

## ARUs

- Assumption: KooshaPari/PhenoCompose is the correct GitHub slug for the badge and upload destination.
- Risk: with no current tests, Codecov upload behavior depends on whether CI later adds at least one Vitest-covered file.
- Uncertainty: no local dependency lockfile exists, so install resolution remains CI-managed.
