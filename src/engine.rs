use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;

use crate::models::{Account, StoredTransaction, Transaction, TransactionType};

/// Transaction processing engine
pub struct PaymentsEngine {
    /// Map of client ID to account
    accounts: HashMap<u16, Account>,
    /// Map of transaction ID to stored disputable transactions (deposits only)
    disputable_transactions: HashMap<u32, StoredTransaction>,
    /// Set of all processed transaction IDs (for duplicate detection)
    processed_tx_ids: HashSet<u32>,
}

impl PaymentsEngine {
    /// Create a new payments engine
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            disputable_transactions: HashMap::new(),
            processed_tx_ids: HashSet::new(),
        }
    }

    /// Process a single transaction
    pub fn process_transaction(&mut self, tx: Transaction) {
        // Check for duplicate transaction ID for deposits and withdrawals only
        // (dispute/resolve/chargeback reference existing transaction IDs)
        if matches!(
            tx.tx_type,
            TransactionType::Deposit | TransactionType::Withdrawal
        ) && self.processed_tx_ids.contains(&tx.tx)
        {
            eprintln!(
                "Warning: Ignoring duplicate transaction ID {} for client {}",
                tx.tx, tx.client
            );
            return;
        }

        // Validate amount for deposit/withdrawal
        if matches!(
            tx.tx_type,
            TransactionType::Deposit | TransactionType::Withdrawal
        ) {
            if let Some(amount) = tx.amount {
                // Reject negative or zero amounts for deposits/withdrawals
                if amount <= Decimal::ZERO {
                    eprintln!(
                        "Warning: Ignoring transaction {} with non-positive amount",
                        tx.tx
                    );
                    return;
                }
            } else {
                eprintln!(
                    "Warning: Ignoring {:?} transaction {} without amount",
                    tx.tx_type, tx.tx
                );
                return;
            }
        }

        let tx_id = tx.tx;
        let tx_type = tx.tx_type;

        match tx_type {
            TransactionType::Deposit => {
                self.process_deposit(tx);
                // Mark deposit transaction ID as processed
                self.processed_tx_ids.insert(tx_id);
            }
            TransactionType::Withdrawal => {
                self.process_withdrawal(tx);
                // Mark withdrawal transaction ID as processed
                self.processed_tx_ids.insert(tx_id);
            }
            TransactionType::Dispute => self.process_dispute(tx),
            TransactionType::Resolve => self.process_resolve(tx),
            TransactionType::Chargeback => self.process_chargeback(tx),
        }
    }

    /// Process a deposit transaction
    fn process_deposit(&mut self, tx: Transaction) {
        let amount = tx.amount.expect("amount validated by process_transaction");

        // Get or create account
        let account = self
            .accounts
            .entry(tx.client)
            .or_insert_with(|| Account::new(tx.client));

        // Process deposit (returns false if account is locked)
        if !account.deposit(amount) {
            eprintln!(
                "Warning: Ignoring deposit on locked account for client {} (tx {})",
                tx.client, tx.tx
            );
            return;
        }

        // Store transaction for potential dispute
        self.disputable_transactions.insert(
            tx.tx,
            StoredTransaction::new(tx.tx, tx.client, amount, TransactionType::Deposit),
        );
    }

    /// Process a withdrawal transaction
    fn process_withdrawal(&mut self, tx: Transaction) {
        let amount = tx.amount.expect("amount validated by process_transaction");

        // Get account (ignore if doesn't exist)
        let account = match self.accounts.get_mut(&tx.client) {
            Some(acc) => acc,
            None => return,
        };

        // Process withdrawal (returns false if insufficient funds or account is locked)
        if !account.withdraw(amount) {
            if account.locked {
                eprintln!(
                    "Warning: Ignoring withdrawal on locked account for client {} (tx {})",
                    tx.client, tx.tx
                );
            } else {
                eprintln!(
                    "Warning: Ignoring withdrawal due to insufficient funds for client {} (tx {})",
                    tx.client, tx.tx
                );
            }
        }
    }

    /// Process a dispute transaction
    fn process_dispute(&mut self, tx: Transaction) {
        // Look up the referenced transaction
        let stored_tx = match self.disputable_transactions.get_mut(&tx.tx) {
            Some(t) => t,
            None => return, // Transaction doesn't exist, ignore
        };

        // Verify client ID matches (security check)
        if stored_tx.client_id != tx.client {
            eprintln!(
                "Warning: Ignoring dispute on transaction {} - client mismatch",
                tx.tx
            );
            return;
        }

        // Check if already disputed
        if stored_tx.disputed {
            eprintln!(
                "Warning: Ignoring dispute on transaction {} - already disputed",
                tx.tx
            );
            return;
        }

        // Get the account
        let account = match self.accounts.get_mut(&tx.client) {
            Some(acc) => acc,
            None => return, // Account doesn't exist, should not happen but handle gracefully
        };

        // Move funds from available to held (returns false if insufficient available)
        if !account.hold(stored_tx.amount) {
            eprintln!(
                "Warning: Ignoring dispute on transaction {} - insufficient available balance (client {})",
                tx.tx, tx.client
            );
            return;
        }

        // Mark transaction as disputed
        stored_tx.disputed = true;
    }

    /// Process a resolve transaction
    fn process_resolve(&mut self, tx: Transaction) {
        // Look up the referenced transaction
        let stored_tx = match self.disputable_transactions.get_mut(&tx.tx) {
            Some(t) => t,
            None => return, // Transaction doesn't exist, ignore
        };

        // Verify client ID matches (security check)
        if stored_tx.client_id != tx.client {
            eprintln!(
                "Warning: Ignoring resolve on transaction {} - client mismatch",
                tx.tx
            );
            return;
        }

        // Check if under dispute
        if !stored_tx.disputed {
            return; // Not under dispute, ignore
        }

        // Get the account
        let account = match self.accounts.get_mut(&tx.client) {
            Some(acc) => acc,
            None => return, // Account doesn't exist, should not happen but handle gracefully
        };

        // Move funds from held back to available (returns false if insufficient held)
        if !account.release(stored_tx.amount) {
            eprintln!(
                "Warning: Ignoring resolve on transaction {} - insufficient held balance (client {})",
                tx.tx, tx.client
            );
            return;
        }

        // Mark transaction as no longer disputed
        stored_tx.disputed = false;
    }

    /// Process a chargeback transaction
    fn process_chargeback(&mut self, tx: Transaction) {
        // Look up the referenced transaction
        let stored_tx = match self.disputable_transactions.get_mut(&tx.tx) {
            Some(t) => t,
            None => return, // Transaction doesn't exist, ignore
        };

        // Verify client ID matches (security check)
        if stored_tx.client_id != tx.client {
            eprintln!(
                "Warning: Ignoring chargeback on transaction {} - client mismatch",
                tx.tx
            );
            return;
        }

        // Check if under dispute
        if !stored_tx.disputed {
            return; // Not under dispute, ignore
        }

        // Get the account
        let account = match self.accounts.get_mut(&tx.client) {
            Some(acc) => acc,
            None => return, // Account doesn't exist, should not happen but handle gracefully
        };

        // Remove held funds and lock account (returns false if insufficient held)
        if !account.chargeback(stored_tx.amount) {
            eprintln!(
                "Warning: Ignoring chargeback on transaction {} - insufficient held balance (client {})",
                tx.tx, tx.client
            );
            return;
        }

        // Mark transaction as no longer disputed (it's been charged back)
        stored_tx.disputed = false;
    }

    /// Get all client accounts
    pub fn get_accounts(&self) -> Vec<&Account> {
        self.accounts.values().collect()
    }

    /// Consume the engine and return all accounts
    pub fn into_accounts(self) -> Vec<Account> {
        self.accounts.into_values().collect()
    }
}

impl Default for PaymentsEngine {
    fn default() -> Self {
        Self::new()
    }
}
