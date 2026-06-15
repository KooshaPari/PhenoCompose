// SPDX-License-Identifier: MIT OR Apache-2.0
//! T71: PhenoCompose hexagonal port — Orchestrator.
//!
//! Defines the `Orchestrator` trait (deploy/rollback/status) and
//! the [`Deployment`] / [`DeployStatus`] value types that flow
//! across the port. Adapters in [`super::adapters`] implement the
//! trait against concrete deployment engines (ArgoCD, Helm,
//! Flux, ...).

use async_trait::async_trait;
use std::collections::HashMap;

/// A deployment request — describes *what* should be deployed.
#[derive(Debug, Clone)]
pub struct Deployment {
    /// Human-readable name of the deployment.
    pub name: String,
    /// Chart (or template) name to deploy.
    pub chart: String,
    /// Free-form key/value overrides for the chart.
    pub values: HashMap<String, String>,
    /// Kubernetes-style namespace (or the equivalent scope
    /// identifier for the underlying engine).
    pub namespace: String,
}

/// The state of a deployment, as reported by
/// [`Orchestrator::status`] and returned by [`Orchestrator::deploy`].
#[derive(Debug, Clone)]
pub struct DeployStatus {
    /// Mirrors [`Deployment::name`].
    pub name: String,
    /// Adapter-defined revision number (Helm revision, ArgoCD
    /// history id, etc.). Adapters should bump this monotonically
    /// per `deploy` so callers can detect changes.
    pub revision: i64,
    /// Short phase string (e.g. `"deployed"`, `"Synced"`,
    /// `"Failed"`, `"Unknown"`). Adapters are free to define
    /// their own vocabulary.
    pub phase: String,
    /// Human-readable message — typically a one-line summary of
    /// the latest operation.
    pub message: String,
}

/// The Orchestrator port trait — `Send + Sync` + no generics + no
/// associated types ⇒ object-safe ⇒ storable as
/// `Box<dyn Orchestrator>`.
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Return a short identifier for the backing engine
    /// (`"argocd"`, `"helm"`, `"flux"`, ...).
    fn backend(&self) -> &str;

    /// Deploy the given [`Deployment`], returning the resulting
    /// [`DeployStatus`].
    async fn deploy(
        &self,
        d: &Deployment,
    ) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>>;

    /// Roll back the named deployment to the given revision.
    async fn rollback(
        &self,
        name: &str,
        revision: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Query the current [`DeployStatus`] for the named
    /// deployment. Adapters that don't know about the deployment
    /// should return a status with `phase = "Unknown"` rather than
    /// an error, so callers can branch on the cause.
    async fn status(
        &self,
        name: &str,
    ) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>>;
}
