use payments_engine::engine::PaymentsEngine;
use payments_engine::models::{Transaction, TransactionType};
use rust_decimal_macros::dec;

// Helper to create a transaction
fn make_transaction(
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<rust_decimal::Decimal>,
) -> Transaction {
    Transaction {
        tx_type,
        client,
        tx,
        amount,
    }
}

#[test]
fn test_engine_creation() {
    let engine = PaymentsEngine::new();
    assert_eq!(engine.get_accounts().len(), 0);
}

#[test]
fn test_basic_deposit_creates_account() {
    let mut engine = PaymentsEngine::new();

    let tx = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(tx);

    let accounts = engine.get_accounts();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].client_id, 1);
    assert_eq!(accounts[0].available, dec!(100));
}

#[test]
fn test_duplicate_deposit_rejected() {
    let mut engine = PaymentsEngine::new();

    // First deposit
    let tx1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(tx1);

    // Duplicate deposit with same tx ID
    let tx2 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(50)));
    engine.process_transaction(tx2);

    let accounts = engine.get_accounts();
    assert_eq!(accounts.len(), 1);
    // Should only have first deposit, not second
    assert_eq!(accounts[0].available, dec!(100));
}

#[test]
fn test_duplicate_withdrawal_rejected() {
    let mut engine = PaymentsEngine::new();

    // Setup: deposit funds
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(200)));
    engine.process_transaction(deposit);

    // First withdrawal
    let tx1 = make_transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(50)));
    engine.process_transaction(tx1);

    // Duplicate withdrawal with same tx ID
    let tx2 = make_transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(50)));
    engine.process_transaction(tx2);

    let accounts = engine.get_accounts();
    // Should only process withdrawal once: 200 - 50 = 150
    assert_eq!(accounts[0].available, dec!(150));
}

#[test]
fn test_negative_deposit_rejected() {
    let mut engine = PaymentsEngine::new();

    let tx = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(-100)));
    engine.process_transaction(tx);

    // No account should be created for invalid transaction
    assert_eq!(engine.get_accounts().len(), 0);
}

#[test]
fn test_zero_deposit_rejected() {
    let mut engine = PaymentsEngine::new();

    let tx = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(0)));
    engine.process_transaction(tx);

    // No account should be created for invalid transaction
    assert_eq!(engine.get_accounts().len(), 0);
}

#[test]
fn test_negative_withdrawal_rejected() {
    let mut engine = PaymentsEngine::new();

    // Setup: deposit funds
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    let withdrawal = make_transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(-50)));
    engine.process_transaction(withdrawal);

    let accounts = engine.get_accounts();
    // Balance should remain unchanged
    assert_eq!(accounts[0].available, dec!(100));
}

#[test]
fn test_deposit_without_amount_rejected() {
    let mut engine = PaymentsEngine::new();

    let tx = make_transaction(TransactionType::Deposit, 1, 1, None);
    engine.process_transaction(tx);

    // No account should be created
    assert_eq!(engine.get_accounts().len(), 0);
}

#[test]
fn test_withdrawal_without_amount_rejected() {
    let mut engine = PaymentsEngine::new();

    // Setup: deposit funds
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    let withdrawal = make_transaction(TransactionType::Withdrawal, 1, 2, None);
    engine.process_transaction(withdrawal);

    let accounts = engine.get_accounts();
    // Balance should remain unchanged
    assert_eq!(accounts[0].available, dec!(100));
}

#[test]
fn test_dispute_nonexistent_transaction_ignored() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // Dispute non-existent transaction
    let dispute = make_transaction(TransactionType::Dispute, 1, 999, None);
    engine.process_transaction(dispute);

    let accounts = engine.get_accounts();
    // Funds should remain available (not held)
    assert_eq!(accounts[0].available, dec!(100));
    assert_eq!(accounts[0].held, dec!(0));
}

#[test]
fn test_cross_client_dispute_rejected() {
    let mut engine = PaymentsEngine::new();

    // Client 1 deposits
    let deposit1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit1);

    // Client 2 deposits
    let deposit2 = make_transaction(TransactionType::Deposit, 2, 2, Some(dec!(200)));
    engine.process_transaction(deposit2);

    // Client 2 tries to dispute Client 1's transaction
    let dispute = make_transaction(TransactionType::Dispute, 2, 1, None);
    engine.process_transaction(dispute);

    let accounts = engine.get_accounts();
    // Find client 1's account
    let client1 = accounts.iter().find(|a| a.client_id == 1).unwrap();
    // Client 1's funds should remain available (dispute should fail)
    assert_eq!(client1.available, dec!(100));
    assert_eq!(client1.held, dec!(0));
}

#[test]
fn test_resolve_without_dispute_ignored() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // Resolve without dispute
    let resolve = make_transaction(TransactionType::Resolve, 1, 1, None);
    engine.process_transaction(resolve);

    let accounts = engine.get_accounts();
    // Funds should remain available
    assert_eq!(accounts[0].available, dec!(100));
    assert_eq!(accounts[0].held, dec!(0));
}

#[test]
fn test_chargeback_without_dispute_ignored() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // Chargeback without dispute
    let chargeback = make_transaction(TransactionType::Chargeback, 1, 1, None);
    engine.process_transaction(chargeback);

    let accounts = engine.get_accounts();
    // Funds should remain and account not locked
    assert_eq!(accounts[0].available, dec!(100));
    assert!(!accounts[0].locked);
}

#[test]
fn test_redispute_same_transaction() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // First dispute
    let dispute1 = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute1);

    // Try to dispute again (should be ignored)
    let dispute2 = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute2);

    let accounts = engine.get_accounts();
    // Should still have 0 available, 100 held (not double-held)
    assert_eq!(accounts[0].available, dec!(0));
    assert_eq!(accounts[0].held, dec!(100));
}

#[test]
fn test_dispute_with_insufficient_available() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // Withdraw most of it
    let withdrawal = make_transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(80)));
    engine.process_transaction(withdrawal);

    // Try to dispute original deposit (need 100 but only 20 available)
    let dispute = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute);

    let accounts = engine.get_accounts();
    // Dispute should fail, funds remain available
    assert_eq!(accounts[0].available, dec!(20));
    assert_eq!(accounts[0].held, dec!(0));
}

#[test]
fn test_multiple_clients_isolated() {
    let mut engine = PaymentsEngine::new();

    // Client 1 transactions
    let tx1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(tx1);

    // Client 2 transactions
    let tx2 = make_transaction(TransactionType::Deposit, 2, 2, Some(dec!(200)));
    engine.process_transaction(tx2);

    // Client 3 transactions
    let tx3 = make_transaction(TransactionType::Deposit, 3, 3, Some(dec!(300)));
    engine.process_transaction(tx3);

    let accounts = engine.get_accounts();
    assert_eq!(accounts.len(), 3);

    // Verify each client's balance is isolated
    for account in accounts {
        match account.client_id {
            1 => assert_eq!(account.available, dec!(100)),
            2 => assert_eq!(account.available, dec!(200)),
            3 => assert_eq!(account.available, dec!(300)),
            _ => panic!("Unexpected client ID"),
        }
    }
}

#[test]
fn test_locked_account_rejects_deposits() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit1);

    // Dispute and chargeback to lock account
    let dispute = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute);

    let chargeback = make_transaction(TransactionType::Chargeback, 1, 1, None);
    engine.process_transaction(chargeback);

    // Try to deposit on locked account
    let deposit2 = make_transaction(TransactionType::Deposit, 1, 2, Some(dec!(50)));
    engine.process_transaction(deposit2);

    let accounts = engine.get_accounts();
    // Account should remain at 0 (chargeback removed funds)
    assert_eq!(accounts[0].available, dec!(0));
    assert!(accounts[0].locked);
}

#[test]
fn test_locked_account_rejects_withdrawals() {
    let mut engine = PaymentsEngine::new();

    // Deposit twice
    let deposit1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit1);

    let deposit2 = make_transaction(TransactionType::Deposit, 1, 2, Some(dec!(50)));
    engine.process_transaction(deposit2);

    // Dispute and chargeback first deposit to lock account
    let dispute = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute);

    let chargeback = make_transaction(TransactionType::Chargeback, 1, 1, None);
    engine.process_transaction(chargeback);

    // Try to withdraw from locked account
    let withdrawal = make_transaction(TransactionType::Withdrawal, 1, 3, Some(dec!(25)));
    engine.process_transaction(withdrawal);

    let accounts = engine.get_accounts();
    // Account should have 50 (second deposit not chargedback)
    assert_eq!(accounts[0].available, dec!(50));
    assert!(accounts[0].locked);
}

#[test]
fn test_dispute_after_chargeback_ignored() {
    let mut engine = PaymentsEngine::new();

    // Deposit
    let deposit = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(deposit);

    // Dispute
    let dispute1 = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute1);

    // Chargeback
    let chargeback = make_transaction(TransactionType::Chargeback, 1, 1, None);
    engine.process_transaction(chargeback);

    // Try to dispute again after chargeback
    let dispute2 = make_transaction(TransactionType::Dispute, 1, 1, None);
    engine.process_transaction(dispute2);

    let accounts = engine.get_accounts();
    // Account should remain at 0 held (dispute after chargeback ignored)
    assert_eq!(accounts[0].available, dec!(0));
    assert_eq!(accounts[0].held, dec!(0));
    assert!(accounts[0].locked);
}

#[test]
fn test_transaction_id_globally_unique() {
    let mut engine = PaymentsEngine::new();

    // Client 1 uses tx ID 1
    let tx1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(tx1);

    // Client 2 tries to use same tx ID 1 (should be rejected - globally unique)
    let tx2 = make_transaction(TransactionType::Deposit, 2, 1, Some(dec!(200)));
    engine.process_transaction(tx2);

    let accounts = engine.get_accounts();

    // Only client 1 should have an account, client 2's duplicate tx should be rejected
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].client_id, 1);
    assert_eq!(accounts[0].available, dec!(100));
}

#[test]
fn test_different_transaction_ids_across_clients() {
    let mut engine = PaymentsEngine::new();

    // Client 1 uses tx ID 1
    let tx1 = make_transaction(TransactionType::Deposit, 1, 1, Some(dec!(100)));
    engine.process_transaction(tx1);

    // Client 2 uses different tx ID 2 (should succeed)
    let tx2 = make_transaction(TransactionType::Deposit, 2, 2, Some(dec!(200)));
    engine.process_transaction(tx2);

    let accounts = engine.get_accounts();
    assert_eq!(accounts.len(), 2);

    // Both transactions should succeed with different IDs
    let client1 = accounts.iter().find(|a| a.client_id == 1).unwrap();
    let client2 = accounts.iter().find(|a| a.client_id == 2).unwrap();

    assert_eq!(client1.available, dec!(100));
    assert_eq!(client2.available, dec!(200));
}
