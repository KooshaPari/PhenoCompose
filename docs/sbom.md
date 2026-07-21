# PhenoCompose SBOM (Software Bill of Materials)

We generate a CycloneDX-format SBOM on every release.

## Generation

```bash
# Install syft
brew install syft  # or: curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin

# Generate SBOM for the workspace
syft scan . -o cyclonedx-json=sbom.json
```

The SBOM is attached to every GitHub release (`.attestation/sbom.json`).

## What we track

- Direct dependencies (Cargo, npm)
- Transitive dependencies (full tree)
- License per package
- Package URL (purl)
- SHA-256 hashes

## License compliance

See `deny.toml` for the full license allow-list. Currently allowed:
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, MPL-2.0
- CC0-1.0 (for docs)
- Unicode-DFS-2016, Unicode-3.0 (for unicode data files)
