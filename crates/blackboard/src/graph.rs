//! Graph relationships and traversal utilities

use crate::types::{AgentId};
use zed42_core::types::{ArtifactId, DecisionId, TaskId};
use serde::{Deserialize, Serialize};


/// Graph edge types representing relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeType {
    SpawnedBy { parent: AgentId, child: AgentId },
    RespondsTo { from: AgentId, to: AgentId },
    Produces { agent: AgentId, artifact: ArtifactId, task: TaskId },
    Implements { artifact: ArtifactId, decision: DecisionId },
    DependsOn { from: ArtifactId, to: ArtifactId },
    Invalidates { new: DecisionId, old: DecisionId },
}

/// Decision Graph structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<EdgeType>,
}
