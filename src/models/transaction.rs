use rust_decimal::Decimal;
use serde::Deserialize;

/// Type of transaction
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Transaction record from CSV input
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_optional_amount")]
    pub amount: Option<Decimal>,
}

/// Custom deserializer to handle empty strings as None for amount field
fn deserialize_optional_amount<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};

    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        Ok(None)
    } else {
        s.parse::<Decimal>().map(Some).map_err(de::Error::custom)
    }
}
