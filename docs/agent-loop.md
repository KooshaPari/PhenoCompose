# PhenoCompose Agent Loop

`phenocompose-agentctl` is a single-binary JSON-in/JSON-out CLI.

## Schema

Request:
```json
{"method": "manifest.parse", "params": {"name": "hello"}}
```

Response:
```json
{"ok": true, "result": {"id": "mf-hello", "name": "hello"}}
```

## Methods

| Method | Params | Returns |
|--------|--------|---------|
| manifest.parse | `{name: string}` | `{id, name}` |
| secret.get | `{id: string}` | `{id, version}` |
| secret.list | `{}` | `[]` |

## Usage

```bash
echo '{"method":"manifest.parse","params":{"name":"hello"}}' | cargo run -p phenocompose-agentctl
```
