use serde::{Deserialize, Serialize};

pub type AgentId = uuid::Uuid;
pub type TeamId = uuid::Uuid;
pub type MessageId = uuid::Uuid;
pub type ThreadId = uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Red,
    Blue,
    Green,
}

pub type Priority = u8;
pub type SessionId = uuid::Uuid;
pub type ArtifactId = String;

pub type DecisionId = String;
pub type TaskId = String;
pub type Confidence = f32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Paused,
    Failed,
    Terminated,
    Laggard,
    Ghost,
}

/// A task assigned to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub context: Option<String>,
    pub constraints: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    /// Create a new task with a description
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            context: None,
            constraints: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add context to the task
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
}

/// An artifact produced by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub task_id: TaskId,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub file_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Type of artifact produced
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Code,
    Test,
    Documentation,
    Diagram,
    Config,
    Other,
}

impl Artifact {
    /// Create a new code artifact
    pub fn code(task_id: TaskId, content: String, file_path: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_id,
            artifact_type: ArtifactType::Code,
            content,
            file_path,
            created_at: chrono::Utc::now(),
        }
    }
}
