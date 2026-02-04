//! ZED42 Agents - Red/Blue/Green Team Implementation
//!
//! Copyright (c) 2025 ZED42 Team. All rights reserved.
//! This software is proprietary. See LICENSE file for terms.
//!
//! Specialized agents with distinct mandates and toolbox assignments.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zed42_core::types::{AgentId, Team, AgentStatus};


pub mod red;
pub mod blue;
pub mod green;
pub mod base;
pub mod orchestrator;
pub mod state;

/// Agent type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentType {
    // Red Team (Offensive)
    PenetrationTester,
    ChaosEngineer,
    PerformanceAnalyst,
    EdgeCaseMiner,
    TechnicalDebtor,

    // Blue Team (Defensive)
    FeatureImplementer,
    Refactorer,
    TestEngineer,
    DocumentationWriter,
    MigrationSpecialist,

    // Green Team (Governance)
    Architect,
    StandardsEnforcer,
    SecurityReviewer,
}

impl AgentType {
    /// Returns the team this agent type belongs to
    pub fn team(&self) -> Team {
        match self {
            AgentType::PenetrationTester
            | AgentType::ChaosEngineer
            | AgentType::PerformanceAnalyst
            | AgentType::EdgeCaseMiner
            | AgentType::TechnicalDebtor => Team::Red,

            AgentType::FeatureImplementer
            | AgentType::Refactorer
            | AgentType::TestEngineer
            | AgentType::DocumentationWriter
            | AgentType::MigrationSpecialist => Team::Blue,

            AgentType::Architect
            | AgentType::StandardsEnforcer
            | AgentType::SecurityReviewer => Team::Green,
        }
    }

    /// Returns the default toolbox assignment for this agent type
    pub fn default_toolbox(&self) -> Vec<String> {
        match self {
            AgentType::PenetrationTester => vec![
                "StaticAnalysis".to_string(),
                "DependencyScanning".to_string(),
                "Fuzzing".to_string(),
                "GraphQuery".to_string(),
            ],
            AgentType::ChaosEngineer => vec![
                "Sandboxing".to_string(),
                "ProcessManagement".to_string(),
                "PerformanceProfiling".to_string(),
            ],
            AgentType::PerformanceAnalyst => vec![
                "PerformanceProfiling".to_string(),
                "MetricsAnalysis".to_string(),
                "GraphQuery".to_string(),
                "VisualizationGeneration".to_string(),
            ],
            AgentType::EdgeCaseMiner => vec![
                "Fuzzing".to_string(),
                "Testing".to_string(),
                "StaticAnalysis".to_string(),
            ],
            AgentType::TechnicalDebtor => vec![
                "MetricsAnalysis".to_string(),
                "StaticAnalysis".to_string(),
                "GraphQuery".to_string(),
                "DiagramGeneration".to_string(),
            ],
            AgentType::FeatureImplementer => vec![
                "CodeGeneration".to_string(),
                "FileManipulation".to_string(),
                "Testing".to_string(),
                "GitOperations".to_string(),
                "BuildSystem".to_string(),
            ],
            AgentType::Refactorer => vec![
                "Refactoring".to_string(),
                "StaticAnalysis".to_string(),
                "Testing".to_string(),
                "GitOperations".to_string(),
            ],
            AgentType::TestEngineer => vec![
                "Testing".to_string(),
                "Fuzzing".to_string(),
                "CodeGeneration".to_string(),
                "PerformanceProfiling".to_string(),
            ],
            AgentType::DocumentationWriter => vec![
                "FileManipulation".to_string(),
                "GitOperations".to_string(),
                "DiagramGeneration".to_string(),
            ],
            AgentType::MigrationSpecialist => vec![
                "CodeGeneration".to_string(),
                "Sandboxing".to_string(),
                "GitOperations".to_string(),
                "BuildSystem".to_string(),
                "Testing".to_string(),
            ],
            AgentType::Architect => vec![
                "GraphQuery".to_string(),
                "MetricsAnalysis".to_string(),
                "DiagramGeneration".to_string(),
                "VisualizationGeneration".to_string(),
                "GitHistory".to_string(),
            ],
            AgentType::StandardsEnforcer => vec![
                "StaticAnalysis".to_string(),
                "MetricsAnalysis".to_string(),
            ],
            AgentType::SecurityReviewer => vec![
                "StaticAnalysis".to_string(),
                "DependencyScanning".to_string(),
                "GraphQuery".to_string(),
                "GitHistory".to_string(),
            ],
        }
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub toolbox: Vec<String>,
    pub spawned_by: Option<AgentId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Agent {
    pub fn new(agent_type: AgentType, spawned_by: Option<AgentId>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_type: agent_type.clone(),
            status: AgentStatus::Idle,
            toolbox: agent_type.default_toolbox(),
            spawned_by,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn team(&self) -> Team {
        self.agent_type.team()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_team_assignment() {
        assert_eq!(AgentType::PenetrationTester.team(), Team::Red);
        assert_eq!(AgentType::FeatureImplementer.team(), Team::Blue);
        assert_eq!(AgentType::Architect.team(), Team::Green);
    }

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(AgentType::FeatureImplementer, None);
        assert_eq!(agent.team(), Team::Blue);
        assert!(!agent.toolbox.is_empty());
    }
}
