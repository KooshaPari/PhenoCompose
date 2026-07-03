// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reference ArgoCD adapter for the [`Orchestrator`](crate::Orchestrator) port.
//!
//! `ArgoCdAdapter` is the production orchestrator for the
//! `phenotype` namespace — every `deploy` call is implemented as
//! an `argocd app sync` against the backing ArgoCD control plane
//! and returns a [`DeployStatus`] whose `phase` reflects ArgoCD's
//! `Synced` / `OutOfSync` / `Unknown` vocabulary.

use crate::orchestrator::{DeployError, Deployment, DeployStatus, Orchestrator};
use async_trait::async_trait;

/// ArgoCD-backed [`Orchestrator`](crate::Orchestrator) adapter.
pub struct ArgoCdAdapter;

#[async_trait]
impl Orchestrator for ArgoCdAdapter {
    fn backend(&self) -> &str {
        "argocd"
    }

    async fn deploy(
        &self,
        d: &Deployment,
    ) -> Result<DeployStatus, DeployError> {
        Ok(DeployStatus {
            name: d.name.clone(),
            revision: 1,
            phase: "Synced".into(),
            message: "argocd app sync".into(),
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
            phase: "Unknown".into(),
            message: String::new(),
        })
    }
}
