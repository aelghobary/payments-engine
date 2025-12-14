use rust_decimal::Decimal;

use super::transaction::TransactionType;

/// Stored transaction for dispute reference
/// Only deposits are stored as they are the only disputable transaction type
#[derive(Debug, Clone)]
pub struct StoredTransaction {
    pub tx_id: u32,
    pub client_id: u16,
    pub amount: Decimal,
    pub tx_type: TransactionType,
    pub disputed: bool,
}

impl StoredTransaction {
    /// Create a new stored transaction
    pub fn new(tx_id: u32, client_id: u16, amount: Decimal, tx_type: TransactionType) -> Self {
        Self {
            tx_id,
            client_id,
            amount,
            tx_type,
            disputed: false,
        }
    }
}
