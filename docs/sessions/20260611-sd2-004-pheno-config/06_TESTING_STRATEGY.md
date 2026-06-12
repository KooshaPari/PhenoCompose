# Testing Strategy

- Verify Git diff matches intended extraction scope.
- Run targeted local checks that do not require network access.
- If local Node dependencies are present, run npm run docs:build; otherwise report the build as unexecuted.
