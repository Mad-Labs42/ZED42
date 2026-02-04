use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Unique identifier for an entity (Agent, Project, User)
pub type EntityId = String;

/// Unique identifier for a temporary lease
pub type LeaseId = String;

/// Usage report from an inference call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input (prompt) tokens
    pub input_tokens: u32,
    /// Number of output (completion) tokens
    pub output_tokens: u32,
    /// Model identifier (e.g., "gpt-4")
    pub model: String,
}

/// Cost rate for a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateTableEntry {
    /// Model identifier
    pub model: String,
    /// Cost per 1000 input tokens
    pub input_cost_per_1k: Decimal,
    /// Cost per 1000 output tokens
    pub output_cost_per_1k: Decimal,
}

/// Status of the budget
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BudgetStatus {
    #[default]
    Active,
    Frozen,
    Depleted,
}

/// Budget configuration and state for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Entity this budget belongs to
    pub entity_id: EntityId,
    /// Hard limit - transactions rejected if exceeded
    pub hard_limit: Decimal,
    /// Soft limit - warnings issued if exceeded
    pub soft_limit: Decimal,
    /// Total amount spent so far
    pub spent: Decimal,
    /// Currency code (e.g., "USD")
    pub currency: String,
    /// Current status of the budget
    #[serde(default)] 
    pub status: BudgetStatus,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Type of ledger transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    /// A reservation of funds (Lease)
    Grant,
    /// A final deduction of funds (Commit)
    Settlement,
    /// Manual adjustment
    Adjustment,
    /// System audit event (Status change, Freeze, etc)
    SystemAudit,
}

/// Immutable record of a financial event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Unique ID of the entry
    #[serde(skip)]
    pub id: Option<String>,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Entity involved
    pub entity_id: EntityId,
    /// Associated Lease ID (if any)
    pub lease_id: Option<LeaseId>,
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Amount reserved or spent
    pub amount: Decimal,
    /// Description or metadata
    pub details: String,
}

/// Receipt returned after committing usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Final cost of the transaction
    pub cost: Decimal,
    /// Remaining budget (hard limit - spent)
    pub remaining_budget: Decimal,
    /// Timestamp of the receipt
    pub timestamp: DateTime<Utc>,
    /// Warning message if soft limit exceeded
    pub warning: Option<String>,
}

/// A temporary reservation of funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lease {
    #[serde(skip)]
    pub id: Option<LeaseId>,
    pub entity_id: EntityId,
    pub estimated_cost: Decimal,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
