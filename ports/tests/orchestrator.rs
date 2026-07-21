// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for the Orchestrator port and its adapters.

use phenocompose_ports::adapters::argocd::ArgoCdAdapter;
use phenocompose_ports::adapters::helm::HelmAdapter;
use phenocompose_ports::{DeployError, Deployment, NoopOrchestrator, Orchestrator};

fn fixture_deployment() -> Deployment {
    let mut values = std::collections::HashMap::new();
    values.insert("replicaCount".to_string(), "3".to_string());
    Deployment {
        name: "phenocommand-web".to_string(),
        chart: "phenocommand-web-0.1.0".to_string(),
        values,
        namespace: "phenotype".to_string(),
    }
}

// ---------------------------------------------------------------------------
// ArgoCD adapter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn argocd_backend_is_argocd() {
    assert_eq!(ArgoCdAdapter.backend(), "argocd");
}

#[tokio::test]
async fn argocd_deploy_reports_synced() {
    let d = fixture_deployment();
    let s = ArgoCdAdapter.deploy(&d).await.unwrap();
    assert_eq!(s.name, d.name);
    assert!(s.phase.to_lowercase().contains("sync"));
    assert!(s.revision >= 1);
}

// ---------------------------------------------------------------------------
// Helm adapter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn helm_backend_is_helm() {
    assert_eq!(HelmAdapter.backend(), "helm");
}

#[tokio::test]
async fn helm_deploy_reports_deployed_with_chart_name() {
    let d = fixture_deployment();
    let s = HelmAdapter.deploy(&d).await.unwrap();
    assert_eq!(s.name, d.name);
    assert_eq!(s.phase, "deployed");
    assert!(s.message.contains(&d.chart));
}

// ---------------------------------------------------------------------------
// Orchestrator object safety
// ---------------------------------------------------------------------------

#[tokio::test]
async fn orchestrator_trait_is_object_safe() {
    // Compile-time check: Orchestrator is object-safe (no
    // associated types, no generic methods).
    fn _takes_dyn(_o: &dyn Orchestrator) {}
    let _argocd: Box<dyn Orchestrator> = Box::new(ArgoCdAdapter);
    let _helm: Box<dyn Orchestrator> = Box::new(HelmAdapter);
    let _noop: Box<dyn Orchestrator> = Box::new(NoopOrchestrator);
}

// ---------------------------------------------------------------------------
// Rollback / status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn adapters_rollback_succeeds() {
    ArgoCdAdapter.rollback("phenocommand-web", 1).await.unwrap();
    HelmAdapter.rollback("phenocommand-web", 1).await.unwrap();
}

#[tokio::test]
async fn adapters_status_returns_unknown_phase_for_unknown_id() {
    let s_argocd = ArgoCdAdapter.status("nonexistent").await.unwrap();
    assert_eq!(s_argocd.phase.to_lowercase(), "unknown");

    let s_helm = HelmAdapter.status("nonexistent").await.unwrap();
    assert_eq!(s_helm.phase.to_lowercase(), "unknown");
}

// ---------------------------------------------------------------------------
// NoopOrchestrator
// ---------------------------------------------------------------------------

#[tokio::test]
async fn noop_orchestrator_backend_is_noop() {
    assert_eq!(NoopOrchestrator.backend(), "noop");
}

#[tokio::test]
async fn noop_orchestrator_deploy_returns_deployed_with_revision_one() {
    let d = fixture_deployment();
    let s = NoopOrchestrator.deploy(&d).await.unwrap();
    assert_eq!(s.name, "phenocommand-web");
    assert_eq!(s.phase, "deployed");
    assert_eq!(s.revision, 1);
    assert!(s.message.contains("noop deploy"));
}

#[tokio::test]
async fn noop_orchestrator_rollback_is_idempotent() {
    NoopOrchestrator.rollback("any-name", 42).await.unwrap();
    NoopOrchestrator.rollback("", 0).await.unwrap();
}

#[tokio::test]
async fn noop_orchestrator_status_returns_deployed() {
    let s = NoopOrchestrator.status("my-app").await.unwrap();
    assert_eq!(s.phase, "deployed");
    assert_eq!(s.revision, 1);
}

#[tokio::test]
async fn noop_orchestrator_deploy_rejects_empty_name() {
    let d = Deployment {
        name: String::new(),
        chart: "chart".to_string(),
        values: std::collections::HashMap::new(),
        namespace: "default".to_string(),
    };
    let err = NoopOrchestrator.deploy(&d).await.unwrap_err();
    assert!(matches!(err, DeployError::Validation(_)));
    assert!(format!("{err}").contains("deploy validation"));
}

#[tokio::test]
async fn noop_orchestrator_status_rejects_empty_name() {
    let err = NoopOrchestrator.status("").await.unwrap_err();
    assert!(matches!(err, DeployError::Validation(_)));
}

#[tokio::test]
async fn noop_orchestrator_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NoopOrchestrator>();
}

// ---------------------------------------------------------------------------
// DeployError
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deploy_error_validation_display() {
    let err = DeployError::validation("empty name");
    let s = format!("{err}");
    assert!(s.contains("deploy validation"));
    assert!(s.contains("empty name"));
}

#[tokio::test]
async fn deploy_error_backend_display() {
    let err = DeployError::backend("argo sync failed");
    let s = format!("{err}");
    assert!(s.contains("deploy backend"));
    assert!(s.contains("argo sync failed"));
}

#[tokio::test]
async fn deploy_error_not_found_display() {
    let err = DeployError::not_found("deployment missing");
    let s = format!("{err}");
    assert!(s.contains("deploy not found"));
    assert!(s.contains("deployment missing"));
}

#[tokio::test]
async fn deploy_error_clone_and_eq() {
    let a = DeployError::validation("bad");
    let b = DeployError::validation("bad");
    assert_eq!(a, b);

    let c = DeployError::backend("fail");
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// NoopOrchestrator as Box<dyn Orchestrator>
// ---------------------------------------------------------------------------

#[tokio::test]
async fn noop_orchestrator_via_dyn_dispatch() {
    let o: Box<dyn Orchestrator> = Box::new(NoopOrchestrator);
    assert_eq!(o.backend(), "noop");

    let d = fixture_deployment();
    let s = o.deploy(&d).await.unwrap();
    assert_eq!(s.phase, "deployed");
}
