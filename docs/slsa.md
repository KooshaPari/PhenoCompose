# PhenoCompose Supply Chain (SLSA L3)

We target [SLSA Level 3](https://slsa.dev/) for our release artifacts.

## Build provenance

Every release is built by a hardened GitHub Actions runner with:
- Isolated build environment (ephemeral VM)
- Reproducible builds (lockfiles pinned)
- Signed provenance attestation (`slsa-framework/slsa-github-generator`)

## Verification

```bash
# Install slsa-verifier
go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@latest

# Verify a release artifact
slsa-verifier verify-artifact phenocompose-x86_64-unknown-linux-gnu.tar.gz \
  --provenance-path phenocompose.intoto.jsonl \
  --source-uri github.com/KooshaPari/PhenoCompose
```

## Tamper resistance

- Releases are signed with `cosign`
- Public key: https://github.com/KooshaPari/PhenoCompose/.well-known/release-signing-key.pub
- Verify: `cosign verify-blob --key release-pub-key.pub phenocompose-x86_64-unknown-linux-gnu.tar.gz`
