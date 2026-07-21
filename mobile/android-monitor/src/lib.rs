//! PhenoCompose Android deploy-monitor companion scaffold.
//!
//! Hosts the Rust half of the JNI bridge between a tiny Kotlin Activity
//! and `pheno-compose-driver`. The companion app polls the driver over
//! loopback and surfaces deploy state as an Android foreground service
//! (PILLAR-TAXONOMY-v2 **L130** — Android-foreground-service).
//!
//! PILLAR-TAXONOMY-v2 cross-references: **L125** (Android Native FFI — jni /
//! ndk) and **L130** (System Service Integration — Android-foreground-service).
//!
//! Status: scaffold only. The Kotlin Activity and the Android manifest land
//! in a follow-up PR alongside the actual JNI bindings.

#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

pub mod jni_bridge;

#[derive(Debug, Error)]
pub enum AndroidMonitorError {
    #[error("JNI bridge unavailable on this target")]
    JniUnavailable,

    #[error("monitor service returned non-zero status: {0}")]
    ServiceFailed(i32),

    #[error("invalid deploy payload: {0}")]
    InvalidPayload(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorCapabilities {
    pub jni_bridge: bool,
    pub foreground_service: bool,
    pub ndk_runtime: bool,
}

impl Default for MonitorCapabilities {
    fn default() -> Self {
        Self {
            jni_bridge: cfg!(target_os = "android"),
            foreground_service: cfg!(target_os = "android"),
            ndk_runtime: cfg!(target_os = "android"),
        }
    }
}

/// Heartbeat ping — the Kotlin Activity calls this on every screen-on
/// event so the Rust side can keep the deploy-state mirror warm.
pub fn ping_heartbeat(seq: u64) -> Result<u64, AndroidMonitorError> {
    #[cfg(all(target_os = "android", feature = "android"))]
    {
        jni_bridge::pheno_monitor_ping(seq).map_err(AndroidMonitorError::ServiceFailed)?;
    }
    Ok(seq.wrapping_add(1))
}

/// Enqueue a deploy-event for the monitor to surface as a notification.
/// On non-Android targets this is a no-op stub.
pub fn push_deploy_event(event_id: &str, severity: u8) -> Result<(), AndroidMonitorError> {
    if event_id.is_empty() {
        return Err(AndroidMonitorError::InvalidPayload(
            "event_id cannot be empty".into(),
        ));
    }
    if severity > 5 {
        return Err(AndroidMonitorError::InvalidPayload(format!(
            "severity {} out of range [0..=5]",
            severity
        )));
    }
    #[cfg(all(target_os = "android", feature = "android"))]
    {
        jni_bridge::pheno_monitor_push(event_id, severity as i32)
            .map_err(AndroidMonitorError::ServiceFailed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_round_trip_on_stub_path() {
        let next = ping_heartbeat(41).expect("stub heartbeat must succeed");
        assert_eq!(next, 42);
    }

    #[test]
    fn push_rejects_empty_event_id() {
        let err = push_deploy_event("", 1).unwrap_err();
        assert!(matches!(err, AndroidMonitorError::InvalidPayload(_)));
    }

    #[test]
    fn push_rejects_out_of_range_severity() {
        let err = push_deploy_event("evt", 9).unwrap_err();
        assert!(matches!(err, AndroidMonitorError::InvalidPayload(_)));
    }

    #[test]
    fn push_succeeds_on_valid_payload() {
        push_deploy_event("deploy-1", 3).expect("valid payload must succeed");
    }
}