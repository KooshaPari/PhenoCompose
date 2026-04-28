# Known Issues

- The repo has no top-level lint script for the docs/TypeScript side, so `lint` focuses on the manifest-backed Go and Rust checks plus the docs build trigger.
- `clean` removes generated artifacts directly; it does not attempt a full dependency cache purge.
