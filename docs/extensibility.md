# PhenoCompose Extensibility

Plugins extend the host application at runtime via a stable dyn-compatible trait.

## Architecture

```
   ┌──────────────────┐
   │  Host (registry) │  <-- PluginRegistry
   └──────────────────┘
            │
            ▼ registers Box<dyn Plugin>
   ┌──────────────────┐    ┌──────────────────┐
   │ SamplePlugin     │    │ JsonBackendPlugin│
   │ (in-process)     │    │ (in-process)     │
   └──────────────────┘    └──────────────────┘
```

## Crates

- `crates/plugin-api/` - the `Plugin` trait + `PluginRegistry` + `PluginInfo`
- `crates/plugin-sample/` - example implementation (`phenotype.plugin.sample`)

## Defining a plugin

```rust
use phenocompose_plugin_api::{Plugin, PluginInfo, PluginRegistry};

pub const ID: &str = "phenotype.plugin.my-plugin";
pub struct MyPlugin { /* state */ }

impl Plugin for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo { id: ID, name: "My Plugin", version: "0.1.0" }
    }
    fn init(&self) -> Result<(), String> {
        eprintln!("[my-plugin] initialized");
        Ok(())
    }
}
```

## Registering

```rust
let mut registry = PluginRegistry::new();
registry.register(MyPlugin { /* ... */ })?;
let p = registry.find("phenotype.plugin.my-plugin").unwrap();
```

## Object-safety

The `Plugin` trait is object-safe (no generics, no `Self` in return types, `&self`/`&mut self` only) and implements `Send + Sync` so plugins can be shared across threads.

## Test

```bash
cargo test -p phenocompose-plugin-sample
# 2 tests, 0.02s
```
