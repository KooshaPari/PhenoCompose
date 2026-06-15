//! Integration tests for the Orchestrator port and its adapters.

use phenocompose_ports::adapters::argocd::ArgoCdAdapter;
use phenocompose_ports::adapters::helm::HelmAdapter;
use phenocompose_ports::{Deployment, Orchestrator};

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

#[tokio::test]
async fn argocd_backend_is_argocd() {
    assert_eq!(ArgoCdAdapter.backend(), "argocd");
}

#[tokio::test]
async fn helm_backend_is_helm() {
    assert_eq!(HelmAdapter.backend(), "helm");
}

#[tokio::test]
async fn argocd_deploy_reports_synced() {
    let d = fixture_deployment();
    let s = ArgoCdAdapter.deploy(&d).await.unwrap();
    assert_eq!(s.name, d.name);
    assert!(s.phase.to_lowercase().contains("sync"));
    assert!(s.revision >= 1);
}

#[tokio::test]
async fn helm_deploy_reports_deployed_with_chart_name() {
    let d = fixture_deployment();
    let s = HelmAdapter.deploy(&d).await.unwrap();
    assert_eq!(s.name, d.name);
    assert_eq!(s.phase, "deployed");
    assert!(s.message.contains(&d.chart));
}

#[tokio::test]
async fn orchestrator_trait_is_object_safe() {
    // Compile-time check: Orchestrator is object-safe (no
    // associated types, no generic methods).
    fn _takes_dyn(_o: &dyn Orchestrator) {}
    let _argocd: Box<dyn Orchestrator> = Box::new(ArgoCdAdapter);
    let _helm: Box<dyn Orchestrator> = Box::new(HelmAdapter);
}

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
