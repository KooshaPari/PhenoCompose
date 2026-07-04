# Android foreground-service scaffold (PILLAR-TAXONOMY-v2 L130)

## Status

Scaffold only. The Kotlin Activity (`PhenoMonitorActivity`), the
foreground service (`PhenoMonitorService`), and the `AndroidManifest.xml`
will be added in a follow-up PR. The Rust crate
(`pheno-android-monitor`) exposes the JNI surface that those Kotlin
classes will call.

## Components (planned)

| File | Role |
| --- | --- |
| `kotlin/PhenoMonitorActivity.kt` | UI shell — surfaces latest deploy state |
| `kotlin/PhenoMonitorService.kt` | Foreground service, holds the Rust runtime |
| `AndroidManifest.xml` | Declares the foreground service + `FOREGROUND_SERVICE_DATA_SYNC` permission |
| `jni/pheno_monitor.c` | C shim called from Kotlin via `external fun` |
| `libpheno_android_monitor.so` | The Rust `cdylib` produced by `cargo build --target aarch64-linux-android` |

## Build (operator runbook)

```bash
# 1. Install the Android NDK + cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk

# 2. Build the Rust cdylib
cargo ndk -t aarch64 -t armv7 -o ./jniLibs build --release -p pheno-android-monitor --features android

# 3. Build the APK (Gradle wrapper will be added in the follow-up PR).
```

## Why this lives in the Rust workspace

The host driver (`pheno-compose-driver`) is a long-running daemon. The
Android companion is a thin status mirror — surfacing deploy state via
notification channel rather than re-implementing orchestration logic.
Putting the JNI half inside the workspace means it can borrow types
directly from `pheno-compose-driver` once the API stabilizes, with no
extra codegen step.