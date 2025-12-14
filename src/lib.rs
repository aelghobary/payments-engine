pub mod engine;
pub mod error;
pub mod models;

use std::io::{Read, Write};

use engine::PaymentsEngine;
use error::Result;

/// Process transactions from a CSV reader and write results to a CSV writer
pub fn process_transactions<R: Read, W: Write>(reader: R, writer: W) -> Result<()> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    let mut engine = PaymentsEngine::new();

    // Process each transaction
    for result in csv_reader.deserialize() {
        match result {
            Ok(transaction) => {
                engine.process_transaction(transaction);
            }
            Err(e) => {
                // Log warning and continue processing
                eprintln!("Warning: Failed to parse transaction: {}", e);
            }
        }
    }

    // Write results
    write_accounts(engine, writer)?;

    Ok(())
}

/// Write client accounts to CSV
fn write_accounts<W: Write>(engine: PaymentsEngine, writer: W) -> Result<()> {
    let mut csv_writer = csv::Writer::from_writer(writer);

    let mut accounts = engine.into_accounts();
    // Sort by client ID for consistent output
    accounts.sort_by_key(|a| a.client_id);

    for account in accounts {
        csv_writer.serialize(account)?;
    }

    csv_writer.flush()?;
    Ok(())
}
