use payments_engine::models::Account;
use rust_decimal_macros::dec;

#[test]
fn test_account_creation() {
    let account = Account::new(1);

    assert_eq!(account.client_id, 1);
    assert_eq!(account.available, dec!(0));
    assert_eq!(account.held, dec!(0));
    assert_eq!(account.total(), dec!(0));
    assert!(!account.locked);
}

#[test]
fn test_deposit_increases_available() {
    let mut account = Account::new(1);

    assert!(account.deposit(dec!(100.50)));

    assert_eq!(account.available, dec!(100.50));
    assert_eq!(account.held, dec!(0));
    assert_eq!(account.total(), dec!(100.50));
}

#[test]
fn test_multiple_deposits() {
    let mut account = Account::new(1);

    assert!(account.deposit(dec!(100)));
    assert!(account.deposit(dec!(50.25)));
    assert!(account.deposit(dec!(25)));

    assert_eq!(account.available, dec!(175.25));
    assert_eq!(account.total(), dec!(175.25));
}

#[test]
fn test_deposit_on_locked_account_fails() {
    let mut account = Account::new(1);
    account.locked = true;

    assert!(!account.deposit(dec!(100)));

    assert_eq!(account.available, dec!(0));
}

#[test]
fn test_withdrawal_decreases_available() {
    let mut account = Account::new(1);
    account.deposit(dec!(200));

    assert!(account.withdraw(dec!(75.50)));

    assert_eq!(account.available, dec!(124.50));
    assert_eq!(account.total(), dec!(124.50));
}

#[test]
fn test_withdrawal_with_insufficient_funds_fails() {
    let mut account = Account::new(1);
    account.deposit(dec!(50));

    assert!(!account.withdraw(dec!(100)));

    // Balance should remain unchanged
    assert_eq!(account.available, dec!(50));
}

#[test]
fn test_withdrawal_on_locked_account_fails() {
    let mut account = Account::new(1);
    account.deposit(dec!(100));
    account.locked = true;

    assert!(!account.withdraw(dec!(50)));

    assert_eq!(account.available, dec!(100));
}

#[test]
fn test_hold_moves_available_to_held() {
    let mut account = Account::new(1);
    account.deposit(dec!(150));

    assert!(account.hold(dec!(100)));

    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(100));
    assert_eq!(account.total(), dec!(150));
}

#[test]
fn test_hold_with_insufficient_available_fails() {
    let mut account = Account::new(1);
    account.deposit(dec!(50));

    assert!(!account.hold(dec!(100)));

    // Balances should remain unchanged
    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(0));
}

#[test]
fn test_hold_exact_available_amount() {
    let mut account = Account::new(1);
    account.deposit(dec!(100));

    assert!(account.hold(dec!(100)));

    assert_eq!(account.available, dec!(0));
    assert_eq!(account.held, dec!(100));
}

#[test]
fn test_release_moves_held_to_available() {
    let mut account = Account::new(1);
    account.deposit(dec!(150));
    account.hold(dec!(100));

    assert!(account.release(dec!(100)));

    assert_eq!(account.available, dec!(150));
    assert_eq!(account.held, dec!(0));
    assert_eq!(account.total(), dec!(150));
}

#[test]
fn test_release_partial_held_amount() {
    let mut account = Account::new(1);
    account.deposit(dec!(150));
    account.hold(dec!(100));

    assert!(account.release(dec!(60)));

    assert_eq!(account.available, dec!(110));
    assert_eq!(account.held, dec!(40));
    assert_eq!(account.total(), dec!(150));
}

#[test]
fn test_release_with_insufficient_held_fails() {
    let mut account = Account::new(1);
    account.deposit(dec!(100));
    account.hold(dec!(50));

    assert!(!account.release(dec!(100)));

    // Balances should remain unchanged
    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(50));
}

#[test]
fn test_chargeback_removes_held_and_locks() {
    let mut account = Account::new(1);
    account.deposit(dec!(150));
    account.hold(dec!(100));

    assert!(account.chargeback(dec!(100)));

    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(0));
    assert_eq!(account.total(), dec!(50));
    assert!(account.locked);
}

#[test]
fn test_chargeback_with_insufficient_held_fails() {
    let mut account = Account::new(1);
    account.deposit(dec!(100));
    account.hold(dec!(50));

    assert!(!account.chargeback(dec!(100)));

    // Balances should remain unchanged and account not locked
    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(50));
    assert!(!account.locked);
}

#[test]
fn test_chargeback_partial_held_amount() {
    let mut account = Account::new(1);
    account.deposit(dec!(200));
    account.hold(dec!(150));

    assert!(account.chargeback(dec!(100)));

    assert_eq!(account.available, dec!(50));
    assert_eq!(account.held, dec!(50));
    assert_eq!(account.total(), dec!(100));
    assert!(account.locked);
}

#[test]
fn test_total_is_sum_of_available_and_held() {
    let mut account = Account::new(1);

    // Initial state
    assert_eq!(account.total(), dec!(0));

    // After deposit
    account.deposit(dec!(200));
    assert_eq!(account.total(), dec!(200));

    // After hold
    account.hold(dec!(75));
    assert_eq!(account.total(), dec!(200));
    assert_eq!(account.available + account.held, dec!(200));

    // After withdrawal
    account.withdraw(dec!(25));
    assert_eq!(account.total(), dec!(175));
    assert_eq!(account.available + account.held, dec!(175));
}

#[test]
fn test_precision_handling() {
    let mut account = Account::new(1);

    // Test 4 decimal precision
    account.deposit(dec!(0.0001));
    assert_eq!(account.available, dec!(0.0001));

    account.deposit(dec!(0.0002));
    assert_eq!(account.available, dec!(0.0003));

    account.withdraw(dec!(0.00015));
    assert_eq!(account.available, dec!(0.00015));
}

#[test]
fn test_locked_account_rejects_all_operations() {
    let mut account = Account::new(1);
    account.deposit(dec!(100));
    account.locked = true;

    // All operations should fail on locked account
    assert!(!account.deposit(dec!(50)));
    assert!(!account.withdraw(dec!(25)));

    assert_eq!(account.available, dec!(100));
}

#[test]
fn test_complex_transaction_sequence() {
    let mut account = Account::new(1);

    // Deposit
    account.deposit(dec!(1000));
    assert_eq!(account.total(), dec!(1000));

    // Withdraw
    account.withdraw(dec!(200));
    assert_eq!(account.total(), dec!(800));

    // Hold some funds
    account.hold(dec!(300));
    assert_eq!(account.available, dec!(500));
    assert_eq!(account.held, dec!(300));

    // Try to withdraw more than available (should fail)
    assert!(!account.withdraw(dec!(600)));
    assert_eq!(account.available, dec!(500));

    // Release held funds
    account.release(dec!(300));
    assert_eq!(account.available, dec!(800));
    assert_eq!(account.held, dec!(0));

    // Now withdrawal should work
    account.withdraw(dec!(600));
    assert_eq!(account.total(), dec!(200));
}
