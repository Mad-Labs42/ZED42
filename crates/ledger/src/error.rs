use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Budget exceeded: hard limit reached for entity {0}")]
    BudgetExceeded(String),

    #[error("Budget frozen: entity {0}")]
    BudgetFrozen(String),

    #[error("Lease not found or expired: {0}")]
    LeaseNotFound(String),

    #[error("Lease already committed: {0}")]
    LeaseAlreadyCommitted(String),

    #[error("Rate not found for model: {0}")]
    RateNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, LedgerError>;
