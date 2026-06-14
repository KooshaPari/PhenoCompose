//! T71: PhenoCompose hexagonal port — Orchestrator.
use async_trait::async_trait;
use std::collections::HashMap;
#[derive(Debug, Clone)] pub struct Deployment { pub name: String, pub chart: String, pub values: HashMap<String, String>, pub namespace: String }
#[derive(Debug, Clone)] pub struct DeployStatus { pub name: String, pub revision: i64, pub phase: String, pub message: String }
#[async_trait]
pub trait Orchestrator: Send + Sync {
    fn backend(&self) -> &str;
    async fn deploy(&self, d: &Deployment) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>>;
    async fn rollback(&self, name: &str, revision: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn status(&self, name: &str) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>>;
}
