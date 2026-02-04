use chrono::Utc;
use rust_decimal_macros::dec;
use surrealdb::engine::any::connect;
use zed42_ledger::{
    error::LedgerError,
    types::{Budget, RateTableEntry, Usage},
    IntelligenceLedger,
};

async fn setup_ledger() -> IntelligenceLedger {
    let db = connect("mem://").await.expect("Failed to connect to memory db");
    db.use_ns("zed42").use_db("ledger").await.expect("Failed to select namespace");
    IntelligenceLedger::new(db)
}

#[tokio::test]
async fn test_ledger_happy_path() {
    let ledger = setup_ledger().await;
    let entity_id = "agent-007";
    let model = "gpt-4";

    // 1. Setup Budget ($10.00) and Rates ($0.03 / 1k input, $0.06 / 1k output)
    ledger.set_budget(Budget {
        entity_id: entity_id.to_string(),
        hard_limit: dec!(10.00),
        soft_limit: dec!(8.00),
        spent: dec!(0.00),
        currency: "USD".to_string(),
        updated_at: Utc::now(),
    }).await.expect("Failed to set budget");

    ledger.set_rate(RateTableEntry {
        model: model.to_string(),
        input_cost_per_1k: dec!(0.03),
        output_cost_per_1k: dec!(0.06),
    }).await.expect("Failed to set rate");

    // 2. Request Lease (Estimate $1.00)
    let lease_id = ledger.request_lease(entity_id, dec!(1.00)).await.expect("Lease denied");

    // 3. Commit Usage (1000 in, 1000 out) -> Should be $0.03 + $0.06 = $0.09
    let usage = Usage {
        input_tokens: 1000,
        output_tokens: 1000,
        model: model.to_string(),
    };

    let receipt = ledger.commit_usage(&lease_id, usage).await.expect("Commit failed");

    // 4. Verify Cost
    assert_eq!(receipt.cost, dec!(0.09));
    assert_eq!(receipt.remaining_budget, dec!(10.00) - dec!(0.09));

    // 5. Verify Budget in DB
    let budget = ledger.get_budget(entity_id).await.expect("DB error").expect("Budget missing");
    assert_eq!(budget.spent, dec!(0.09));
}

#[tokio::test]
async fn test_hard_cap_rejection() {
    let ledger = setup_ledger().await;
    let entity_id = "broke-agent";

    // Budget: $1.00, Spent: $0.95
    ledger.set_budget(Budget {
        entity_id: entity_id.to_string(),
        hard_limit: dec!(1.00),
        soft_limit: dec!(0.80),
        spent: dec!(0.95),
        currency: "USD".to_string(),
        updated_at: Utc::now(),
    }).await.expect("Failed to set budget");

    // Request $0.06 (Total would be $1.01) -> Should Reject
    let result = ledger.request_lease(entity_id, dec!(0.06)).await;
    
    match result {
        Err(LedgerError::BudgetExceeded(_)) => assert!(true),
        _ => panic!("Should have rejected lease due to hard cap"),
    }
}

#[tokio::test]
async fn test_soft_cap_warning() {
    let ledger = setup_ledger().await;
    let entity_id = "warn-agent";
    let model = "gpt-4";

    // Budget: $10.00, Soft: $5.00, Spent: $4.95
    ledger.set_budget(Budget {
        entity_id: entity_id.to_string(),
        hard_limit: dec!(10.00),
        soft_limit: dec!(5.00),
        spent: dec!(4.95),
        currency: "USD".to_string(),
        updated_at: Utc::now(),
    }).await.expect("Failed to set budget");

    ledger.set_rate(RateTableEntry {
        model: model.to_string(),
        input_cost_per_1k: dec!(1.00), 
        output_cost_per_1k: dec!(1.00),
    }).await.expect("Failed to set rate");

    // Request $0.10 (Total $5.05) -> Lease Granted (Under Hard Cap)
    let lease_id = ledger.request_lease(entity_id, dec!(0.10)).await.expect("Lease denied");

    // Commit Usage ($0.10)
    let usage = Usage {
        input_tokens: 50, // $0.05
        output_tokens: 50, // $0.05
        model: model.to_string(),
    };

    let receipt = ledger.commit_usage(&lease_id, usage).await.expect("Commit failed");

    // Verify Warning
    assert!(receipt.warning.is_some());
    assert!(receipt.warning.unwrap().contains("Soft limit exceeded"));
    assert_eq!(receipt.remaining_budget, dec!(10.00) - dec!(5.05));
}

#[tokio::test]
async fn test_precision_math() {
    let ledger = setup_ledger().await;
    let entity_id = "math-agent";
    let model = "cheap-model";

    ledger.set_budget(Budget {
        entity_id: entity_id.to_string(),
        hard_limit: dec!(100.0000),
        soft_limit: dec!(50.0000),
        spent: dec!(0.0000),
        currency: "USD".to_string(),
        updated_at: Utc::now(),
    }).await.expect("Failed to set budget");

    // Rate: $0.0001 per 1k input
    ledger.set_rate(RateTableEntry {
        model: model.to_string(),
        input_cost_per_1k: dec!(0.0001), 
        output_cost_per_1k: dec!(0.0001),
    }).await.expect("Failed to set rate");

    let lease_id = ledger.request_lease(entity_id, dec!(0.10)).await.expect("Lease denied");

    let usage = Usage {
        input_tokens: 1500, // 1.5 * 0.0001 = 0.00015
        output_tokens: 0,
        model: model.to_string(),
    };

    let receipt = ledger.commit_usage(&lease_id, usage).await.expect("Commit failed");

    // 0.00015 cost
    assert_eq!(receipt.cost, dec!(0.00015));
    
    let budget = ledger.get_budget(entity_id).await.expect("DB error").unwrap();
    assert_eq!(budget.spent, dec!(0.00015));
}
