# Research

- Existing CI is in \`.github/workflows/ci.yml\` and already uses pinned SHAs for core GitHub actions.
- \`SECURITY.md\` already states that PhenoCompose is scanned continuously with \`osv-scanner\` across lockfiles, so CI should enforce that policy.
- Existing scheduled security workflows run on Monday in the 04:00 UTC hour; this change uses a non-conflicting Monday slot.

