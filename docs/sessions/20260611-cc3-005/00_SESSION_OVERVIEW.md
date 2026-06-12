# CC3-005 Session Overview

- Goal: add a \`Taskfile.yml\` that exposes \`test\`, \`test-unit\`, and \`test-coverage\` for the repo's VitePress and Vitest workflow.
- Scope: root-level task runner and package manifest updates only.
- Success criteria: \`task test\`, \`task test-unit\`, and \`task test-coverage\` resolve to installable root Node scripts without changing existing \`justfile\` behavior.
