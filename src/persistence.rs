use crate::error::Result;
use crate::models::Transaction;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Persistence backend for crash recovery
///
/// This trait defines the interface for persisting transactions to durable storage.
/// A production implementation would use Write-Ahead Logging (WAL):
///
/// # WAL (Write-Ahead Log) Pattern
///
/// 1. **Write First**: Before processing a transaction in memory, write it to a durable log
/// 2. **Process**: Then process the transaction in the in-memory engine
/// 3. **Crash Recovery**: On restart, replay all transactions from the log to rebuild state
///
/// This ensures that no committed transaction is lost, even if the process crashes
/// immediately after processing.
///
/// # Production Implementation
///
/// A real implementation would:
/// - Open an append-only log file (e.g., `transactions.log`)
/// - Serialize each transaction to JSON or binary format
/// - Write to file buffer
/// - Call `fsync()` to ensure data is on disk (durability)
/// - On recovery, read the entire log file and deserialize transactions
///
/// # Example
///
/// ```no_run
/// use payments_engine::persistence::{PersistenceBackend, StubPersistence};
/// use payments_engine::models::{Transaction, TransactionType};
/// use rust_decimal_macros::dec;
///
/// let mut persistence = StubPersistence::new();
///
/// let tx = Transaction {
///     tx_type: TransactionType::Deposit,
///     client: 1,
///     tx: 1,
///     amount: Some(dec!(100.0)),
/// };
///
/// // In production, this would write to disk + fsync
/// persistence.append(&tx).unwrap();
///
/// // On crash recovery, this would read from disk
/// let transactions = persistence.replay().unwrap();
/// ```
pub trait PersistenceBackend: Send + Sync {
    /// Append a transaction to persistent storage
    ///
    /// # Production Behavior
    ///
    /// 1. Serialize transaction to bytes (JSON, bincode, etc.)
    /// 2. Append to log file
    /// 3. Call `fsync()` to ensure durability
    /// 4. Return error if I/O fails
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to persist
    ///
    /// # Returns
    ///
    /// `Ok(())` if persisted successfully, `Err` if I/O fails
    fn append(&mut self, tx: &Transaction) -> Result<()>;

    /// Replay all transactions from persistent storage
    ///
    /// # Production Behavior
    ///
    /// 1. Open log file
    /// 2. Read and deserialize all transactions
    /// 3. Return them in order for replay
    ///
    /// This is called during crash recovery to rebuild engine state.
    ///
    /// # Returns
    ///
    /// Vector of all transactions in the log, in order
    fn replay(&self) -> Result<Vec<Transaction>>;
}

/// Stub persistence implementation for demonstration
///
/// This implementation demonstrates the persistence interface without actual file I/O.
/// It logs what a production implementation would do, making it useful for:
/// - Understanding the persistence pattern
/// - Testing the integration without file dependencies
/// - Demonstrating crash recovery architecture
///
/// # What It Does
///
/// - `append()`: Logs that a transaction would be persisted, increments counter
/// - `replay()`: Logs that transactions would be replayed, returns empty vec
///
/// # What Production Would Do
///
/// ```text
/// StubPersistence::append()
///   Production:
///     1. let json = serde_json::to_string(tx)?;
///     2. writeln!(log_file, "{}", json)?;
///     3. log_file.sync_all()?;  // fsync - critical for durability
///
/// StubPersistence::replay()
///   Production:
///     1. let file = File::open("transactions.log")?;
///     2. let reader = BufReader::new(file);
///     3. reader.lines()
///            .map(|line| serde_json::from_str(&line?))
///            .collect()
/// ```
///
/// # Example
///
/// ```
/// use payments_engine::persistence::{PersistenceBackend, StubPersistence};
/// use payments_engine::models::{Transaction, TransactionType};
/// use rust_decimal_macros::dec;
///
/// let mut persistence = StubPersistence::new();
///
/// let tx = Transaction {
///     tx_type: TransactionType::Deposit,
///     client: 1,
///     tx: 1,
///     amount: Some(dec!(100.0)),
/// };
///
/// // Logs what would be persisted
/// persistence.append(&tx).unwrap();
///
/// // Logs that replay would happen
/// let transactions = persistence.replay().unwrap();
/// assert_eq!(transactions.len(), 0); // Stub returns empty
/// ```
pub struct StubPersistence {
    /// Counter for tracking how many transactions would be persisted
    /// Uses atomic for thread safety (could be shared across tasks)
    transaction_count: AtomicUsize,
}

impl StubPersistence {
    /// Create a new stub persistence backend
    ///
    /// # Example
    ///
    /// ```
    /// use payments_engine::persistence::StubPersistence;
    ///
    /// let persistence = StubPersistence::new();
    /// ```
    pub fn new() -> Self {
        Self {
            transaction_count: AtomicUsize::new(0),
        }
    }

    /// Get the number of transactions that would have been persisted
    ///
    /// Useful for testing and demonstration purposes
    pub fn transaction_count(&self) -> usize {
        self.transaction_count.load(Ordering::Relaxed)
    }
}

impl Default for StubPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceBackend for StubPersistence {
    fn append(&mut self, tx: &Transaction) -> Result<()> {
        self.transaction_count.fetch_add(1, Ordering::Relaxed);

        // Suppress unused variable warnings
        let _ = tx;

        // Production would:
        // let json = serde_json::to_string(tx)?;
        // writeln!(self.log_file, "{}", json)?;
        // self.log_file.sync_all()?;  // Critical: ensure on disk

        Ok(())
    }

    fn replay(&self) -> Result<Vec<Transaction>> {
        // Production would:
        // let file = File::open(&self.log_path)?;
        // let reader = BufReader::new(file);
        // let transactions: Vec<Transaction> = reader
        //     .lines()
        //     .filter_map(|line| line.ok())
        //     .filter_map(|line| serde_json::from_str(&line).ok())
        //     .collect();
        // Ok(transactions)

        Ok(Vec::new()) // Stub returns empty - simulates fresh start
    }
}
