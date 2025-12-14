use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

/// Account state
#[derive(Debug, Clone)]
pub struct Account {
    pub client_id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

impl Account {
    /// Create a new client account with zero balances
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
        }
    }

    /// Get the total balance (available + held)
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// Deposit funds to available balance
    /// Returns true if successful, false if account is locked
    pub fn deposit(&mut self, amount: Decimal) -> bool {
        if self.locked {
            return false;
        }
        self.available += amount;
        true
    }

    /// Withdraw funds from available balance
    /// Returns true if successful, false if insufficient funds or account is locked
    pub fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.locked {
            return false;
        }
        if self.available < amount {
            return false;
        }
        self.available -= amount;
        true
    }

    /// Move funds from available to held (for dispute)
    /// Returns true if successful, false if insufficient available funds
    pub fn hold(&mut self, amount: Decimal) -> bool {
        if self.available < amount {
            return false;
        }
        self.available -= amount;
        self.held += amount;
        true
    }

    /// Move funds from held back to available (for resolve)
    /// Returns true if successful, false if insufficient held funds
    pub fn release(&mut self, amount: Decimal) -> bool {
        if self.held < amount {
            return false;
        }
        self.held -= amount;
        self.available += amount;
        true
    }

    /// Remove held funds and lock account (for chargeback)
    /// Returns true if successful, false if insufficient held funds
    pub fn chargeback(&mut self, amount: Decimal) -> bool {
        if self.held < amount {
            return false;
        }
        self.held -= amount;
        self.locked = true;
        true
    }
}

// Custom serialization to include computed total field for CSV output
#[derive(Serialize)]
struct AccountSerialized {
    #[serde(rename = "client")]
    client_id: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapper = AccountSerialized {
            client_id: self.client_id,
            available: self.available,
            held: self.held,
            total: self.total(), // Compute on-the-fly
            locked: self.locked,
        };
        wrapper.serialize(serializer)
    }
}
