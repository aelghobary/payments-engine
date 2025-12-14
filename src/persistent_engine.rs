use crate::engine::PaymentsEngine;
use crate::error::Result;
use crate::models::Transaction;
use crate::persistence::PersistenceBackend;

/// Engine with persistence support for crash recovery
///
/// This wrapper implements the Write-Ahead Log (WAL) pattern to ensure durability:
///
/// # WAL Pattern Guarantees
///
/// 1. **Write First**: Transaction is written to persistent storage BEFORE processing
/// 2. **Process**: Transaction is processed in memory
/// 3. **Crash Safety**: If crash happens after step 1, transaction will be replayed on recovery
///
/// This ensures that committed transactions survive crashes.
///
/// # Recovery Process
///
/// On startup, call `PersistentEngine::recover()` instead of `new()`:
/// 1. Replays all transactions from persistent storage
/// 2. Rebuilds in-memory state
/// 3. Continues normal operation
///
/// # Example
///
/// ```no_run
/// use payments_engine::persistent_engine::PersistentEngine;
/// use payments_engine::persistence::StubPersistence;
/// use payments_engine::models::{Transaction, TransactionType};
/// use rust_decimal_macros::dec;
///
/// // Normal startup (fresh state)
/// let mut engine = PersistentEngine::new(StubPersistence::new());
///
/// // Process transactions
/// let tx = Transaction {
///     tx_type: TransactionType::Deposit,
///     client: 1,
///     tx: 1,
///     amount: Some(dec!(100.0)),
/// };
/// engine.process_transaction(tx).unwrap();
///
/// // After crash, recover from persistent storage
/// let recovered = PersistentEngine::recover(StubPersistence::new()).unwrap();
/// // State is restored from WAL
/// ```
///
/// # Thread Safety
///
/// PersistentEngine is NOT thread-safe by itself. For concurrent access, wrap it in:
/// - `Arc<RwLock<PersistentEngine<P>>>` for single-threaded persistence
/// - Use with `ShardedEngine` for high-concurrency scenarios
pub struct PersistentEngine<P: PersistenceBackend> {
    /// In-memory transaction processing engine
    engine: PaymentsEngine,
    /// Persistence backend (WAL)
    persistence: P,
}

impl<P: PersistenceBackend> PersistentEngine<P> {
    /// Create a new engine with persistence backend
    ///
    /// Starts with empty state. Use `recover()` to restore from crash.
    ///
    /// # Arguments
    ///
    /// * `persistence` - Persistence backend implementation
    ///
    /// # Example
    ///
    /// ```
    /// use payments_engine::persistent_engine::PersistentEngine;
    /// use payments_engine::persistence::StubPersistence;
    ///
    /// let engine = PersistentEngine::new(StubPersistence::new());
    /// ```
    pub fn new(persistence: P) -> Self {
        Self {
            engine: PaymentsEngine::new(),
            persistence,
        }
    }

    /// Recover from crash by replaying WAL
    ///
    /// # Recovery Steps
    ///
    /// 1. Create fresh engine
    /// 2. Replay all transactions from persistent storage
    /// 3. Rebuild in-memory state
    /// 4. Return recovered engine ready for normal operation
    ///
    /// # Arguments
    ///
    /// * `persistence` - Persistence backend to replay from
    ///
    /// # Returns
    ///
    /// Recovered engine with state restored
    ///
    /// # Example
    ///
    /// ```no_run
    /// use payments_engine::persistent_engine::PersistentEngine;
    /// use payments_engine::persistence::StubPersistence;
    ///
    /// // Simulate crash recovery
    /// let engine = PersistentEngine::recover(StubPersistence::new()).unwrap();
    /// // Engine state is now restored from WAL
    /// ```
    pub fn recover(persistence: P) -> Result<Self> {
        let mut engine = PaymentsEngine::new();
        let transactions = persistence.replay()?;

        for tx in transactions.iter() {
            engine.process_transaction(tx.clone());
        }

        Ok(Self {
            engine,
            persistence,
        })
    }

    /// Process a transaction with durability guarantee
    ///
    /// # WAL Pattern Implementation
    ///
    /// 1. **Write to WAL first** (crash before this = transaction lost, client should retry)
    /// 2. **Process in memory** (crash after step 1 = transaction will replay on recovery)
    ///
    /// This ensures no committed transaction is lost.
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to process
    ///
    /// # Returns
    ///
    /// `Ok(())` if persisted and processed, `Err` if persistence fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use payments_engine::persistent_engine::PersistentEngine;
    /// use payments_engine::persistence::StubPersistence;
    /// use payments_engine::models::{Transaction, TransactionType};
    /// use rust_decimal_macros::dec;
    ///
    /// let mut engine = PersistentEngine::new(StubPersistence::new());
    ///
    /// let tx = Transaction {
    ///     tx_type: TransactionType::Deposit,
    ///     client: 1,
    ///     tx: 1,
    ///     amount: Some(dec!(100.0)),
    /// };
    ///
    /// engine.process_transaction(tx).unwrap();
    /// ```
    pub fn process_transaction(&mut self, tx: Transaction) -> Result<()> {
        // CRITICAL: Persist BEFORE processing (WAL pattern)
        // This ensures we can recover if we crash after this point
        self.persistence.append(&tx)?;

        // Safe to process now - if we crash, transaction is in WAL
        self.engine.process_transaction(tx);

        Ok(())
    }

    /// Get reference to inner engine for queries
    ///
    /// Useful for read-only operations like getting accounts.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use payments_engine::persistent_engine::PersistentEngine;
    /// use payments_engine::persistence::StubPersistence;
    ///
    /// let engine = PersistentEngine::new(StubPersistence::new());
    /// let accounts = engine.engine().get_accounts();
    /// ```
    pub fn engine(&self) -> &PaymentsEngine {
        &self.engine
    }

    /// Get mutable reference to persistence backend
    ///
    /// Advanced use cases like triggering snapshots.
    pub fn persistence_mut(&mut self) -> &mut P {
        &mut self.persistence
    }
}
