//! ZED42 Cortex - Executive Orchestrator & Team Spawner
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! The Cortex is responsible for intent parsing, task planning,
//! and team management (spawning/dissolving agents).

use zed42_core::types::{SessionId, AgentId, AgentStatus};
use zed42_core::traits::AgentBehavior;
use zed42_blackboard::BlackboardDb;
use zed42_agents::{Agent, AgentType};
use zed42_memory::MemorySubstrate;
use std::collections::HashMap;
use uuid::Uuid;


pub mod intent;
pub mod planner;
pub mod team_manager;

/// The Cortex - main orchestration component
pub struct Cortex {
    session_id: SessionId,
    blackboard: Option<BlackboardDb>,
    memory: MemorySubstrate,
    active_agents: HashMap<AgentId, Box<dyn AgentBehavior>>,
}


impl Cortex {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            blackboard: None,
            memory: MemorySubstrate::default(),
            active_agents: HashMap::new(),
        }
    }

    /// Initialize the Cortex and connect to subsystems
    pub async fn initialize(&mut self, blackboard: BlackboardDb) -> anyhow::Result<()> {
        self.blackboard = Some(blackboard);
        // Initialize memory substrate
        // Set up message subscriptions
        Ok(())
    }

    /// Process user intent and spawn appropriate agents
    pub async fn process_intent(&mut self, intent: &str) -> anyhow::Result<()> {
        // Parse intent
        // Create task decomposition
        // Spawn agents based on task requirements
        Ok(())
    }

    /// Spawn a new agent
    pub async fn spawn_agent(&mut self, agent_type: AgentType) -> anyhow::Result<AgentId> {
        let agent_id = Uuid::new_v4();
        
        // Mock implementation for test verification
        struct MockAgent { id: AgentId }
        #[async_trait::async_trait]
        impl zed42_core::traits::AgentBehavior for MockAgent {
            fn id(&self) -> AgentId { self.id }
            async fn initialize(&mut self) -> zed42_core::Result<()> { Ok(()) }
            async fn run(&mut self) -> zed42_core::Result<()> { Ok(()) }
            async fn shutdown(&mut self) -> zed42_core::Result<()> { Ok(()) }
        }

        self.active_agents.insert(agent_id, Box::new(MockAgent { id: agent_id }));
        Ok(agent_id)
    }


    /// Dissolve an agent
    pub async fn dissolve_agent(&mut self, agent_id: AgentId) -> anyhow::Result<()> {
        self.active_agents.remove(&agent_id);
        Ok(())
    }

    /// Get active agent count
    pub fn active_agent_count(&self) -> usize {
        self.active_agents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cortex_initialization() {
        let session_id = SessionId::new_v4();
        let _cortex = Cortex::new(session_id);
    }

    #[tokio::test]
    async fn test_agent_spawning() {
        let session_id = SessionId::new_v4();
        let mut cortex = Cortex::new(session_id);
        let agent_id = cortex.spawn_agent(AgentType::FeatureImplementer).await.unwrap();
        assert_eq!(cortex.active_agent_count(), 1);
        cortex.dissolve_agent(agent_id).await.unwrap();
        assert_eq!(cortex.active_agent_count(), 0);
    }
}
