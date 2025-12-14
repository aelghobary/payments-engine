use std::env;
use std::fs::File;
use std::io;

use anyhow::{Context, Result};
use payments_engine::process_transactions;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    anyhow::ensure!(
        args.len() == 2,
        "Usage: {} <input.csv>",
        args.first().unwrap_or(&"payments-engine".to_string())
    );

    let filename = &args[1];

    let file = File::open(filename)
        .with_context(|| format!("Failed to open input file '{}'", filename))?;

    process_transactions(file, io::stdout())
        .context("Failed to process transactions and write output")?;

    Ok(())
}
