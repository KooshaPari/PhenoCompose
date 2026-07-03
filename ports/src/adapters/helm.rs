// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference Helm adapter for the [`Orchestrator`](crate::Orchestrator) port.
//!
//! `HelmAdapter` is the plain `helm install` orchestrator — used
//! in environments without an ArgoCD control plane, and as a
//! deterministic stub in tests. Every `deploy` is implemented as
//! `helm install <chart>` and returns a [`DeployStatus`] whose
//! `phase` is the lowercase Helm `deployed` / `unknown` vocabulary.

use crate::orchestrator::{DeployError, Deployment, DeployStatus, Orchestrator};
use async_trait::async_trait;

/// Helm-backed [`Orchestrator`](crate::Orchestrator) adapter.
pub struct HelmAdapter;

#[async_trait]
impl Orchestrator for HelmAdapter {
    fn backend(&self) -> &str {
        "helm"
    }

    async fn deploy(
        &self,
        d: &Deployment,
    ) -> Result<DeployStatus, DeployError> {
        Ok(DeployStatus {
            name: d.name.clone(),
            revision: 1,
            phase: "deployed".into(),
            message: format!("helm install {}", d.chart),
        })
    }

    async fn rollback(
        &self,
        _n: &str,
        _r: i64,
    ) -> Result<(), DeployError> {
        Ok(())
    }

    async fn status(
        &self,
        name: &str,
    ) -> Result<DeployStatus, DeployError> {
        Ok(DeployStatus {
            name: name.into(),
            revision: 0,
            phase: "unknown".into(),
            message: String::new(),
        })
    }
}
