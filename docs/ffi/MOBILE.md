# Mobile / Native FFI — PhenoCompose

PILLAR-TAXONOMY-v2 lines **L121** (macOS), **L122** (iOS), **L125** (Android),
and **L130** (System Service Integration) tracked here.

PhenoCompose is a Rust orchestrator workspace; it currently exposes
`pheno-compose-driver` as a long-running daemon binary. The mobile
scaffolds introduced in PR-A give that daemon a native FFI surface on
macOS and Android so the operator can monitor deploys without going
through a web UI.

## Layout

| Path | Crate | Target | Purpose |
| --- | --- | --- | --- |
| `mobile/macos-shell/` | `pheno-macos-shell` | `aarch64-apple-darwin`, `x86_64-apple-darwin` | AppKit bridge via `swift-rs`, menu-bar host, launchd plist host |
| `mobile/android-monitor/` | `pheno-android-monitor` | `aarch64-linux-android`, `armv7-linux-androideabi` | JNI bridge to a Kotlin foreground-service companion |

Both crates ship as `lib + staticlib + cdylib` so they can be consumed by
Swift (`import PhenoMacosShell`) and by the Kotlin Activity
(`System.loadLibrary("pheno_android_monitor")`) without an extra FFI
codegen step.

## Feature flags

The mobile crates are designed to **compile on linux-x86_64** even
though their platform-specific FFI is gated:

| Crate | Default features | Native feature |
| --- | --- | --- |
| `pheno-macos-shell` | `[]` | `swift` — pulls in `swift-rs = "0.1"` |
| `pheno-android-monitor` | `[]` | `android` — pulls in `ndk = "0.8"` + `ndk-sys = "0.5"` |

This lets `cargo check --workspace --all-targets` pass in CI on linux
without an Apple or Android toolchain. Cross-compile smoke tests are
documented below but are **not CI-gated** in PR-A.

## Cross-compile matrix

| Target | Status | Notes |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | ✅ scaffold passes `cargo check` | default CI target |
| `aarch64-apple-darwin` | 🔄 needs Swift toolchain | enabled by `--features swift` |
| `x86_64-apple-darwin` | 🔄 needs Swift toolchain | enabled by `--features swift` |
| `aarch64-linux-android` | 🔄 needs Android NDK | enabled by `--features android` |
| `armv7-linux-androideabi` | 🔄 needs Android NDK | enabled by `--features android` |

### Build matrix recipes

```bash
# macOS shell-extension cdylib
cargo build --release --target aarch64-apple-darwin \
    -p pheno-macos-shell --features swift

# Android monitor cdylib (via cargo-ndk)
cargo install cargo-ndk
cargo ndk -t armv7 -t aarch64 -t x86_64 -t x86 \
    -o ./mobile/android-monitor/jniLibs \
    build --release -p pheno-android-monitor --features android
```

## Public API summary

### `pheno-macos-shell`

| Item | Where |
| --- | --- |
| `boot_shell_extension() -> Result<ShellCapabilities, MacosShellError>` | `src/lib.rs:60` |
| `notify_deploy_event(event_id, payload) -> Result<(), MacosShellError>` | `src/lib.rs:73` |
| `launchd::install_plist(driver_binary) -> Result<PathBuf, _>` | `src/launchd.rs:78` |
| `launchd::render_plist(driver_binary) -> String` | `src/launchd.rs:55` |
| `LAUNCHD_LABEL = "ai.phenotype.pheno-compose-driver"` | `src/launchd.rs:18` |

### `pheno-android-monitor`

| Item | Where |
| --- | --- |
| `ping_heartbeat(seq) -> Result<u64, AndroidMonitorError>` | `src/lib.rs:50` |
| `push_deploy_event(event_id, severity) -> Result<(), _>` | `src/lib.rs:64` |
| `MonitorCapabilities` (default — all true on Android, all false elsewhere) | `src/lib.rs:32` |

## Audit context

Source: `PhenoCompose-audit.json` (lines 16, 41, 62–66, 82, 241, 244,
295–297). Pre-PR FFI scores: iOS = 0/100 (L122), Android = 0.5/100
(L125). PR-A introduces minimum-viable scaffolding to start raising
those scores; full implementations (Keychain integration, AppKit tray,
WidgetKit, ARKit, full JNI surface) land in subsequent PRs once the FFI
shape is validated.

## Out of scope (follow-up PRs)

- **iOS** (L122) — separate `mobile/ios-shell/` crate, shares Swift sources with macOS via SwiftPM
- **Linux tray** (L124) — `zbus` D-Bus integration, out of scope until a desktop distro is chosen
- **Spotlight / AppleEvents** — Phase 3 of L121
- **ABI-versioning** — see PILLAR-TAXONOMY-v2 L127 (FFI Build & Toolchain)