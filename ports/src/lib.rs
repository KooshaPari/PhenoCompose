// SPDX-License-Identifier: MIT OR Apache-2.0
//! `phenocompose-ports`
//!
//! The Orchestrator port — the canonical hex-architecture port for
//! declarative deployment of [`Deployment`]s via a backing engine
//! (ArgoCD, Helm, Flux, ...). The trait is intentionally
//! transport-agnostic; adapters in [`adapters`] bridge to local
//! deployment engines.
//!
//! Object-safety: the trait has no associated types, no generic
//! methods, and only `&self` receivers (with `Send + Sync`
//! super-traits), so it can be stored as `Box<dyn Orchestrator>`
//! and dispatched dynamically.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod orchestrator;

pub mod adapters;

pub use orchestrator::{DeployStatus, Deployment, Orchestrator};
