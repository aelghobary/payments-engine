use payments_engine::models::{Transaction, TransactionType};
use rust_decimal::Decimal;

/// Helper to create a transaction with all fields
pub fn make_transaction(
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
) -> Transaction {
    Transaction {
        tx_type,
        client,
        tx,
        amount,
    }
}

/// Helper to create a deposit transaction
pub fn make_deposit(client: u16, tx: u32, amount: Decimal) -> Transaction {
    make_transaction(TransactionType::Deposit, client, tx, Some(amount))
}

/// Helper to create a dispute transaction
pub fn make_dispute(client: u16, tx: u32) -> Transaction {
    make_transaction(TransactionType::Dispute, client, tx, None)
}

/// Process a CSV string through the engine and return the output
pub fn process_csv_string(csv_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    payments_engine::process_transactions(csv_input.as_bytes(), &mut output)?;
    Ok(String::from_utf8(output)?)
}

/// Assert that the output contains a client with specific balance values
/// Handles both "0" and "0.0" formats flexibly
pub fn assert_client_balance(
    output: &str,
    client_id: u16,
    available: &str,
    held: &str,
    total: &str,
    locked: bool,
) {
    let locked_str = if locked { "true" } else { "false" };

    // Helper to generate all possible decimal formats for a number
    let format_variants = |num: &str| -> Vec<String> {
        let mut variants = vec![num.to_string()];
        // If it doesn't already have a decimal point, add .0 variant
        if !num.contains('.') {
            variants.push(format!("{}.0", num));
        }
        variants
    };

    let available_variants = format_variants(available);
    let held_variants = format_variants(held);
    let total_variants = format_variants(total);

    // Generate all possible combinations
    let mut found = false;
    for av in &available_variants {
        for hv in &held_variants {
            for tv in &total_variants {
                let pattern = format!("{},{},{},{},{}", client_id, av, hv, tv, locked_str);
                if output.contains(&pattern) {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if found {
            break;
        }
    }

    assert!(
        found,
        "Expected output to contain client {} with available={}, held={}, total={}, locked={}\nActual output:\n{}",
        client_id, available, held, total, locked_str, output
    );
}

/// Create a test CSV from a list of transaction descriptions
pub fn build_csv(transactions: &[(&str, u16, u32, &str)]) -> String {
    let mut csv = String::from("type,client,tx,amount\n");

    for (tx_type, client, tx, amount) in transactions {
        csv.push_str(&format!("{},{},{},{}\n", tx_type, client, tx, amount));
    }

    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_make_deposit() {
        let tx = make_deposit(1, 100, dec!(50.25));
        assert_eq!(tx.client, 1);
        assert_eq!(tx.tx, 100);
        assert_eq!(tx.amount, Some(dec!(50.25)));
        assert!(matches!(tx.tx_type, TransactionType::Deposit));
    }

    #[test]
    fn test_make_dispute() {
        let tx = make_dispute(1, 100);
        assert_eq!(tx.client, 1);
        assert_eq!(tx.tx, 100);
        assert_eq!(tx.amount, None);
        assert!(matches!(tx.tx_type, TransactionType::Dispute));
    }

    #[test]
    fn test_build_csv() {
        let csv = build_csv(&[
            ("deposit", 1, 1, "100.0"),
            ("withdrawal", 1, 2, "50.0"),
            ("dispute", 1, 1, ""),
        ]);

        assert!(csv.starts_with("type,client,tx,amount\n"));
        assert!(csv.contains("deposit,1,1,100.0"));
        assert!(csv.contains("withdrawal,1,2,50.0"));
        assert!(csv.contains("dispute,1,1,"));
    }

    #[test]
    fn test_process_csv_string() {
        let csv = "type,client,tx,amount\ndeposit,1,1,100.0\n";
        let output = process_csv_string(csv).unwrap();

        assert!(output.contains("client,available,held,total,locked"));
        assert!(output.contains("1,100"));
    }
}
