// SPDX-License-Identifier: MIT OR Apache-2.0
//! PhenoCompose hexagonal port — Orchestrator.
//!
//! Defines the [`Orchestrator`] trait (deploy/rollback/status), the
//! [`Deployment`] / [`DeployStatus`] value types, a [`DeployError`]
//! error type consistent with the sibling port crates, and a
//! [`NoopOrchestrator`] stub for tests and dry-run modes.
//!
//! Adapters in [`super::adapters`] implement the trait against
//! concrete deployment engines (ArgoCD, Helm, Flux, ...).

use async_trait::async_trait;
use phenocompose_port_types::PortError;
use std::collections::HashMap;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Value types
// ---------------------------------------------------------------------------

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
/// [`Orchestrator::status`] and returned by
/// [`Orchestrator::deploy`].
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

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors an [`Orchestrator`] can return.
///
/// This is a thin wrapper around [`PortError`] that adds the
/// `#[non_exhaustive]` attribute (so we can add variants in
/// future minor versions without a SemVer break) and a private
/// `Backend` variant carrying the adapter's own error type
/// rendered as a `String`.
///
/// Consistent with the sibling port error types
/// ([`ComposeError`](phenocompose_port_composer::ComposeError),
/// [`PublishError`](phenocompose_port_publisher::PublishError),
/// [`RuntimeError`](phenocompose_port_runtime::RuntimeError),
/// [`SecretStoreError`](phenocompose_port_secret::SecretStoreError)).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum DeployError {
    /// The deployment request failed validation before any
    /// backend work happened (e.g. an empty name).
    #[error("deploy validation: {0}")]
    Validation(String),
    /// The backend (argo, helm, flux, ...) failed.
    /// The wrapped `String` is the backend's own error message.
    #[error("deploy backend: {0}")]
    Backend(String),
    /// The deployment or revision was not found (returned by
    /// [`Orchestrator::rollback`] or [`Orchestrator::status`]
    /// when the target does not exist).
    #[error("deploy not found: {0}")]
    NotFound(String),
}

impl DeployError {
    /// Convenience constructor for [`DeployError::Validation`].
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Convenience constructor for [`DeployError::Backend`].
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }

    /// Convenience constructor for [`DeployError::NotFound`].
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl From<PortError> for DeployError {
    fn from(e: PortError) -> Self {
        match e {
            PortError::Validation(s) | PortError::Unsupported(s) => Self::Validation(s),
            PortError::NotFound(s) => Self::NotFound(s),
            PortError::Transport(s) => Self::Backend(s),
        }
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// The Orchestrator port trait — `Send + Sync` + no generics + no
/// associated types ⇒ object-safe ⇒ storable as
/// `Box<dyn Orchestrator>`.
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Return a short identifier for the backing engine
    /// (`"argocd"`, `"helm"`, `"flux"`, `"noop"`, ...).
    fn backend(&self) -> &str;

    /// Deploy the given [`Deployment`], returning the resulting
    /// [`DeployStatus`].
    ///
    /// # Errors
    ///
    /// Returns [`DeployError::Validation`] for malformed
    /// deployments, [`DeployError::Backend`] for adapter
    /// failures.
    async fn deploy(&self, d: &Deployment) -> Result<DeployStatus, DeployError>;

    /// Roll back the named deployment to the given revision.
    ///
    /// # Errors
    ///
    /// Returns [`DeployError::NotFound`] if the deployment or
    /// revision does not exist, [`DeployError::Backend`] for
    /// adapter failures.
    async fn rollback(&self, name: &str, revision: i64) -> Result<(), DeployError>;

    /// Query the current [`DeployStatus`] for the named
    /// deployment.
    ///
    /// Adapters that don't know about the deployment should
    /// return a status with `phase = "Unknown"` rather than
    /// an error, so callers can branch on the cause.
    ///
    /// # Errors
    ///
    /// Returns [`DeployError::Backend`] for adapter failures.
    async fn status(&self, name: &str) -> Result<DeployStatus, DeployError>;
}

// ---------------------------------------------------------------------------
// NoopOrchestrator
// ---------------------------------------------------------------------------

/// A trivial in-memory [`Orchestrator`] used for tests and as a
/// default for adapters that deploy nowhere (e.g. a dry-run mode
/// that just logs what would be deployed).
///
/// `NoopOrchestrator` mirrors the pattern used by
/// [`NoopComposer`](phenocompose_port_composer::NoopComposer),
/// [`NoopPublisher`](phenocompose_port_publisher::NoopPublisher),
/// [`NoopRuntime`](phenocompose_port_runtime::NoopRuntime), and
/// [`NoopSecretStore`](phenocompose_port_secret::NoopSecretStore)
/// — every sibling port crate provides a stub implementation
/// that always succeeds (or returns a deterministic response)
/// for use in tests and dry-run modes.
///
/// # Behaviour
///
/// | Method       | Behaviour                                                          |
/// |--------------|--------------------------------------------------------------------|
/// | `deploy`     | Returns `DeployStatus` with `phase = "deployed"`, `revision = 1`   |
/// | `rollback`   | Always `Ok(())` (idempotent)                                       |
/// | `status`     | Always returns `phase = "deployed"`, `revision = 1`                |
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopOrchestrator;

#[async_trait]
impl Orchestrator for NoopOrchestrator {
    fn backend(&self) -> &str {
        "noop"
    }

    async fn deploy(&self, d: &Deployment) -> Result<DeployStatus, DeployError> {
        validate_deployment(d)?;
        Ok(DeployStatus {
            name: d.name.clone(),
            revision: 1,
            phase: "deployed".into(),
            message: format!("noop deploy {}", d.chart),
        })
    }

    async fn rollback(&self, _name: &str, _revision: i64) -> Result<(), DeployError> {
        Ok(())
    }

    async fn status(&self, name: &str) -> Result<DeployStatus, DeployError> {
        if name.is_empty() {
            return Err(DeployError::validation("deployment name is empty"));
        }
        Ok(DeployStatus {
            name: name.into(),
            revision: 1,
            phase: "deployed".into(),
            message: String::new(),
        })
    }
}

/// Validate a [`Deployment`] for minimum required fields.
/// Returns `Ok(())` if the deployment is structurally valid.
pub(crate) fn validate_deployment(d: &Deployment) -> Result<(), DeployError> {
    if d.name.is_empty() {
        return Err(DeployError::validation("deployment name is empty"));
    }
    if d.chart.is_empty() {
        return Err(DeployError::validation("deployment chart is empty"));
    }
    Ok(())
}
