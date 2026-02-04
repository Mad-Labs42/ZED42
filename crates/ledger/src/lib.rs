//! Intelligence Ledger - Financial Governance for AI Agents
//!
//! A standalone, double-entry bookkeeping system for tracking token usage and costs.

pub mod error;
pub mod types;

use crate::error::{LedgerError, Result};
use crate::types::*;
use chrono::Utc;
use rust_decimal::prelude::*;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use uuid::Uuid;
use zed42_core::ledger::BudgetStatus;

/// The Intelligence Ledger - Single source of truth for financial data
#[derive(Clone)]
pub struct IntelligenceLedger {
    db: Surreal<Any>,
    /// Table name for budgets
    table_budgets: String,
    /// Table name for rate tables
    table_rates: String,
    /// Table name for ledger entries
    table_ledger: String,
    /// Table name for active leases
    table_leases: String,
}

impl IntelligenceLedger {
    /// Create a new Intelligence Ledger instance
    pub fn new(db: Surreal<Any>) -> Self {
        Self {
            db,
            table_budgets: "budgets".to_string(),
            table_rates: "rate_table".to_string(),
            table_ledger: "ledger_entries".to_string(),
            table_leases: "leases".to_string(),
        }
    }

    /// Initialize a budget for an entity
    pub async fn set_budget(&self, budget: Budget) -> Result<()> {
        let _: Option<Budget> = self
            .db
            .update((&self.table_budgets, &budget.entity_id))
            .content(budget)
            .await?;
        Ok(())
    }

    /// Set the cost rate for a model
    pub async fn set_rate(&self, rate: RateTableEntry) -> Result<()> {
        let _: Option<RateTableEntry> = self
            .db
            .update((&self.table_rates, &rate.model))
            .content(rate)
            .await?;
        Ok(())
    }

    /// Request a lease for an estimated cost
    ///
    /// # Arguments
    /// - `entity_id` - The entity requesting funds
    /// - `estimated_cost` - The projected maximum cost
    ///
    /// # Returns
    /// - `LeaseId` - A unique reservation token
    ///
    /// # Errors
    /// - `BudgetExceeded` - If hard limit would be breached
    pub async fn request_lease(
        &self,
        entity_id: &str,
        estimated_cost: Decimal,
    ) -> Result<LeaseId> {
        // 1. Fetch Budget
        let budget: Option<Budget> = self.db.select((&self.table_budgets, entity_id)).await?;
        let budget = budget.ok_or_else(|| {
            LedgerError::BudgetExceeded(format!("No budget found for entity {}", entity_id))
        })?;

        // 1b. Check Status
        if budget.status != BudgetStatus::Active {
            return Err(LedgerError::BudgetFrozen(entity_id.to_string()));
        }

        // 2. Check Hard Cap
        if budget.spent + estimated_cost > budget.hard_limit {
            return Err(LedgerError::BudgetExceeded(entity_id.to_string()));
        }

        // 3. Create Lease
        let lease_id = Uuid::new_v4().to_string();
        let lease = Lease {
            id: None, // Let DB manage the ID mapping
            entity_id: entity_id.to_string(),
            estimated_cost,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5), // 5 min TTL
        };

        let _: Option<Lease> = self
            .db
            .create((&self.table_leases, &lease_id))
            .content(lease)
            .await?;

        // 4. Record Grant Entry (Immutable)
        let entry = LedgerEntry {
            id: None,
            timestamp: Utc::now(),
            entity_id: entity_id.to_string(),
            lease_id: Some(lease_id.clone()),
            transaction_type: TransactionType::Grant,
            amount: estimated_cost,
            details: format!("Lease requested for {}", estimated_cost),
        };

        let _: Option<LedgerEntry> = self.db.create(&self.table_ledger).content(entry).await?;

        Ok(lease_id)
    }

    /// Commit actual usage and settle the lease
    ///
    /// # Arguments
    /// - `lease_id` - The reservation token
    /// - `usage` - The actua token usage
    ///
    /// # Returns
    /// - `Receipt` - Final cost and remaining budget
    pub async fn commit_usage(&self, lease_id: &str, usage: Usage) -> Result<Receipt> {
        // 1. Retrieve Lease
        let lease: Option<Lease> = self.db.select((&self.table_leases, lease_id)).await?;
        let lease = lease.ok_or_else(|| LedgerError::LeaseNotFound(lease_id.to_string()))?;

        // 2. Get Rate
        let rate: Option<RateTableEntry> =
            self.db.select((&self.table_rates, &usage.model)).await?;
        let rate = rate
            .ok_or_else(|| LedgerError::RateNotFound(usage.model.clone()))?;

        // 3. Calculate Actual Cost
        let input_cost = (Decimal::from(usage.input_tokens) / Decimal::from(1000))
            * rate.input_cost_per_1k;
        let output_cost = (Decimal::from(usage.output_tokens) / Decimal::from(1000))
            * rate.output_cost_per_1k;
        let actual_cost = input_cost + output_cost;

        // 4. Update Budget (Atomic Increment logic simulated here, explicitly safe in single-writer or careful locking)
        // SurrealDB supports "UPDATE type::thing($tb, $id) SET spent += $amount" logic
        // But for type safety we fetch-update for now in this MVP or use a merge query
        
        // Let's use a merge query for atomicity if possible, or just strict update
        // We need the new budget value anyway
        
        let mut budget: Budget = self
             .db
             .select((&self.table_budgets, &lease.entity_id))
             .await?
             .ok_or_else(|| LedgerError::BudgetExceeded("Budget missing during commit".to_string()))?;
        
        budget.spent += actual_cost;
        budget.updated_at = Utc::now();
        
        let _: Option<Budget> = self
            .db
            .update((&self.table_budgets, &lease.entity_id))
            .content(budget.clone())
            .await?;

        // 5. Record Settlement Entry
        let entry = LedgerEntry {
            id: None,
            timestamp: Utc::now(),
            entity_id: lease.entity_id.clone(),
            lease_id: Some(lease_id.to_string()),
            transaction_type: TransactionType::Settlement,
            amount: actual_cost,
            details: format!(
                "Usage committed: {} in / {} out on {}",
                usage.input_tokens, usage.output_tokens, usage.model
            ),
        };
        let _: Option<LedgerEntry> = self.db.create(&self.table_ledger).content(entry).await?;

        // 6. Close Lease (Delete)
        let _: Option<Lease> = self.db.delete((&self.table_leases, lease_id)).await?;

        // 7. Check Soft Cap
        let warning = if budget.spent > budget.soft_limit {
            Some(format!(
                "Soft limit exceeded: {} > {}",
                budget.spent, budget.soft_limit
            ))
        } else {
            None
        };

        Ok(Receipt {
            cost: actual_cost,
            remaining_budget: budget.hard_limit - budget.spent,
            timestamp: Utc::now(),
            warning,
        })
    }
    
    /// Freeze a budget, preventing further leases
    pub async fn freeze_budget(&self, entity_id: &str, reason: &str) -> Result<()> {
        let mut budget: Budget = self
             .db
             .select((&self.table_budgets, entity_id))
             .await?
             .ok_or_else(|| LedgerError::BudgetExceeded("Budget missing during freeze".to_string()))?;
        
        budget.status = BudgetStatus::Frozen;
        budget.updated_at = Utc::now();
        
        let _: Option<Budget> = self
            .db
            .update((&self.table_budgets, entity_id))
            .content(budget)
            .await?;
            
        // Log System Audit
        let entry = LedgerEntry {
            id: None,
            timestamp: Utc::now(),
            entity_id: entity_id.to_string(),
            lease_id: None,
            transaction_type: TransactionType::SystemAudit,
            amount: Decimal::default(),
            details: format!("Budget Frozen: {}", reason),
        };
        let _: Option<LedgerEntry> = self.db.create(&self.table_ledger).content(entry).await?;
        
        Ok(())
    }

    /// Get current budget for an entity
    pub async fn get_budget(&self, entity_id: &str) -> Result<Option<Budget>> {
        Ok(self.db.select((&self.table_budgets, entity_id)).await?)
    }
}
