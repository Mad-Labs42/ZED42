use async_trait::async_trait;
use crate::{AgentId, Result};

#[async_trait]
pub trait AgentBehavior: Send + Sync {
    fn id(&self) -> AgentId;
    async fn initialize(&mut self) -> Result<()>;
    async fn run(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}
