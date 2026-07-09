# PhenoCompose Fuzzing

cargo-fuzz targets for value-type parsing.

## Running

```bash
cargo install cargo-fuzz
cargo fuzz --manifest-path /Users/kooshapari/CodeProjects/PhenoCompose-clean/fuzz run manifest_parse
cargo fuzz --manifest-path /Users/kooshapari/CodeProjects/PhenoCompose-clean/fuzz run secret_ref
```

## Targets

- `manifest_parse` - fuzzes Manifest name parsing
- `secret_ref` - fuzzes SecretRef construction (rejects invalid input)
