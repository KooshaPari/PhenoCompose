# Specifications

- Add weekly \`schedule\` trigger to CI.
- Add a dedicated dependency audit job.
- Run \`npm audit --audit-level=high\`.
- Run \`osv-scanner\` recursively against the repository after dependency installation.

## ARUs

- Assumption: \`google/osv-scanner-action@v2\` accepts \`scan-args\` with recursive repository scanning.
- Risk: upstream action interface drift could require a follow-up pin/update.
- Uncertainty: local execution cannot fully validate remote GitHub Action behavior without GitHub-hosted runners.

