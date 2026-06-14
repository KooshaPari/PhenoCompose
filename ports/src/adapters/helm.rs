use super::orchestrator::{Deployment, DeployStatus, Orchestrator};
use async_trait::async_trait;
pub struct HelmAdapter;
#[async_trait]
impl Orchestrator for HelmAdapter {
    fn backend(&self) -> &str { "helm" }
    async fn deploy(&self, d: &Deployment) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DeployStatus { name: d.name.clone(), revision: 1, phase: "deployed".into(), message: format!("helm install {}", d.chart) })
    }
    async fn rollback(&self, _n: &str, _r: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
    async fn status(&self, name: &str) -> Result<DeployStatus, Box<dyn std::error::Error + Send + Sync>> { Ok(DeployStatus { name: name.into(), revision: 0, phase: "unknown".into(), message: "".into() }) }
}
