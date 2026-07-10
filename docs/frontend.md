# PhenoCompose Frontend (Web UI)

A Dioxus-based SPA for the ops console.

## Stack

- `dioxus` v0.5+ - Rust-native SPA/web/desktop framework
- `dioxus-router` - client-side routing
- `dioxus-hooks` - state management
- `reqwest` - HTTP client
- `web-sys` - browser bindings

## Why Dioxus?

- Single language (Rust) across web + desktop
- Same React-like mental model
- Compile-time checks for HTML correctness
- Small bundle size vs. Electron or Tauri

## Components

```
   ┌──────────────────────────────────┐
   │ <App>                            │
   │   <Nav />                       │
   │   <Routes>                      │
   │     <Route path="/" Dashboard/> │
   │     <Route path="/sandbox" />  │
   │     <Route path="/secret" />   │
   │   </Routes>                     │
   └──────────────────────────────────┘
```

## State

```rust
#[derive(Clone)]
struct AppState {
    sandboxes: Signal<Vec<Sandbox>>,
    secrets: Signal<Vec<Secret>>,
    user: Signal<Option<User>>,
}
```

## Routes

| Path | Component | Backend |
|------|-----------|---------|
| `/` | `Dashboard` | list running sandboxes, recent events |
| `/sandbox` | `SandboxList` | filter by status, user, image |
| `/sandbox/:id` | `SandboxDetail` | logs, metrics, exec |
| `/secret` | `SecretList` | filter by namespace, name |
| `/secret/:ns/:name` | `SecretDetail` | value (masked), versions, audit |
| `/admin` | `AdminPanel` | audit log, plugin registry, config |

## Build

```bash
dx build --release --platform web
# Output: dist/ (static SPA)
```

## Bundle size target

- Initial JS: < 200KB gzipped
- Total CSS: < 50KB
- No external font dependencies

## Accessibility

- ARIA labels on all interactive elements
- Color contrast ratio >= 4.5:1
- Keyboard navigation: all controls
- Screen reader friendly: aria-live for log streams
