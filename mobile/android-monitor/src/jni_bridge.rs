//! JNI bridge — only compiled when targeting Android with the `android`
//! feature enabled. On every other target the module is empty so the
//! scaffold type-checks on linux/Windows.
//!
//! PILLAR-TAXONOMY-v2 **L125** (Android Native FFI — jni / ndk).

#![cfg(all(target_os = "android", feature = "android"))]

/// Stub JNI entry point — receives a sequence number from the Kotlin
/// Activity and returns the next expected sequence. Real implementation
/// will use `jni::JNIEnv` to grab a global `Context` reference and
/// register a `ServiceConnection` for `PhenoMonitorService`.
pub fn pheno_monitor_ping(seq: u64) -> Result<(), i32> {
    let _ = seq;
    Ok(())
}

/// Stub JNI entry point — forwards a deploy-event to the foreground
/// service notification surface.
pub fn phono_monitor_push(event_id: &str, severity: i32) -> Result<(), i32> {
    let _ = (event_id, severity);
    Ok(())
}