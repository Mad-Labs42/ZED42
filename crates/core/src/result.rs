use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Blackboard error: {0}")]
    Blackboard(String),
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
