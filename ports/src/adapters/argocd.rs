use super::orchestrator::{Deployment, DeployStatus, Orchestrator};
use async_trait::async_trait;
pub struct ArgoCdAdapter;
#[async_trait]
impl Orchestrator for ArgoCdAdapter {
    fn backend(&self) -> &str { "argocd" }
    async fn deploy(&self, d: &Deployment) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>> { Ok(DeployStatus { name: d.name.clone(), revision: 1, phase: "Synced".into(), message: "argocd app sync".into() }) }
    async fn rollback(&self, _n: &str, _r: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    async fn status(&self, name: &str) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>> { Ok(DeployStatus { name: name.into(), revision: 0, phase: "Unknown".into(), message: "".into() }) }
}
