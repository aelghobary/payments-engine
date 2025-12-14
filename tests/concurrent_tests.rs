use payments_engine::concurrent_engine::ShardedEngine;
use payments_engine::models::{Transaction, TransactionType};
use rust_decimal_macros::dec;

/// Test concurrent deposits to the same client
/// Verifies that concurrent transactions are handled correctly
#[tokio::test]
async fn test_concurrent_deposits_same_client() {
    let engine = ShardedEngine::new(4);

    // Spawn 100 concurrent tasks depositing to the same client
    let mut handles = vec![];

    for i in 0..100 {
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: i,
            amount: Some(dec!(10.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for h in handles {
        h.await.unwrap();
    }

    // Verify total balance
    let account = engine.get_account(1).await.unwrap();
    assert_eq!(account.available, dec!(1000.0)); // 100 × 10
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.total(), dec!(1000.0));
}

/// Test concurrent deposits to different clients
/// Demonstrates parallel processing across shards
#[tokio::test]
async fn test_concurrent_deposits_different_clients() {
    let engine = ShardedEngine::new(8);

    // Process 100 clients concurrently
    let mut handles = vec![];

    for client_id in 1..=100 {
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx: client_id as u32,
            amount: Some(dec!(100.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    // Wait for all
    for h in handles {
        h.await.unwrap();
    }

    // Verify all accounts
    let accounts = engine.get_all_accounts().await;
    assert_eq!(accounts.len(), 100);

    for account in accounts {
        assert_eq!(account.available, dec!(100.0));
    }
}

/// Test concurrent mixed operations (deposits, withdrawals, disputes)
/// Simulates realistic workload
#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let engine = ShardedEngine::new(4);

    // First, deposit to client 1
    let tx = Transaction {
        tx_type: TransactionType::Deposit,
        client: 1,
        tx: 1,
        amount: Some(dec!(1000.0)),
    };
    engine.process_transaction(tx).await.unwrap();

    // Now spawn concurrent operations
    let mut handles = vec![];

    // 50 withdrawals
    for i in 0..50 {
        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 100 + i,
            amount: Some(dec!(10.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    // 50 deposits to different client
    for i in 0..50 {
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 2,
            tx: 200 + i,
            amount: Some(dec!(20.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    // Verify results
    let account1 = engine.get_account(1).await.unwrap();
    assert_eq!(account1.available, dec!(500.0)); // 1000 - 50×10

    let account2 = engine.get_account(2).await.unwrap();
    assert_eq!(account2.available, dec!(1000.0)); // 50×20
}

/// Test dispute workflow with concurrency
#[tokio::test]
async fn test_concurrent_dispute_workflow() {
    let engine = ShardedEngine::new(4);

    // Deposit transactions for multiple clients
    for client_id in 1..=10 {
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx: client_id as u32,
            amount: Some(dec!(200.0)),
        };
        engine.process_transaction(tx).await.unwrap();
    }

    // Concurrently dispute half of them
    let mut handles = vec![];

    for client_id in 1..=5 {
        let tx = Transaction {
            tx_type: TransactionType::Dispute,
            client: client_id,
            tx: client_id as u32,
            amount: None,
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    // Verify disputed accounts have funds held
    for client_id in 1..=5 {
        let account = engine.get_account(client_id).await.unwrap();
        assert_eq!(account.available, dec!(0.0));
        assert_eq!(account.held, dec!(200.0));
    }

    // Verify non-disputed accounts are untouched
    for client_id in 6..=10 {
        let account = engine.get_account(client_id).await.unwrap();
        assert_eq!(account.available, dec!(200.0));
        assert_eq!(account.held, dec!(0.0));
    }
}

/// Test high concurrency - simulates thousands of transactions
#[tokio::test]
async fn test_high_concurrency() {
    let engine = ShardedEngine::new(16);

    let num_transactions = 1000;
    let mut handles = vec![];

    for i in 0..num_transactions {
        let client_id = (i % 100) + 1; // 100 different clients

        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx: i as u32,
            amount: Some(dec!(1.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    // Wait for all
    for h in handles {
        h.await.unwrap();
    }

    // Each client should have 10 deposits (1000 / 100)
    for client_id in 1..=100 {
        let account = engine.get_account(client_id).await.unwrap();
        assert_eq!(account.available, dec!(10.0));
    }
}

/// Test that sharding doesn't break transaction ordering per client
#[tokio::test]
async fn test_transaction_ordering_per_client() {
    let engine = ShardedEngine::new(4);

    // Deposit
    let tx1 = Transaction {
        tx_type: TransactionType::Deposit,
        client: 1,
        tx: 1,
        amount: Some(dec!(100.0)),
    };

    // Withdrawal
    let tx2 = Transaction {
        tx_type: TransactionType::Withdrawal,
        client: 1,
        tx: 2,
        amount: Some(dec!(30.0)),
    };

    // Dispute
    let tx3 = Transaction {
        tx_type: TransactionType::Dispute,
        client: 1,
        tx: 1,
        amount: None,
    };

    // Process concurrently (but all go to same shard, so serialized)
    let engine1 = engine.clone_handle();
    let engine2 = engine.clone_handle();
    let engine3 = engine.clone_handle();

    let h1 = tokio::spawn(async move { engine1.process_transaction(tx1).await });
    let h2 = tokio::spawn(async move { engine2.process_transaction(tx2).await });
    let h3 = tokio::spawn(async move { engine3.process_transaction(tx3).await });

    h1.await.unwrap().unwrap();
    h2.await.unwrap().unwrap();
    h3.await.unwrap().unwrap();

    // Final state depends on ordering, but should be consistent
    let account = engine.get_account(1).await.unwrap();

    // Possible outcomes:
    // 1. deposit → withdraw → dispute: available=0, held=100
    // 2. deposit → dispute → withdraw: available=0, held=100 (withdraw fails)
    // etc.

    // Key: total should be 100 or 70 (depending on if withdraw happened)
    assert!(
        account.total() == dec!(100.0) || account.total() == dec!(70.0),
        "Total should be 100 or 70, got {}",
        account.total()
    );
}

/// Benchmark-style test to show throughput
#[tokio::test]
async fn test_throughput_demonstration() {
    let engine = ShardedEngine::new(8);

    let num_transactions = 10_000;
    let start = std::time::Instant::now();

    let mut handles = vec![];

    for i in 0..num_transactions {
        let client_id = (i % 1000) + 1;

        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: client_id,
            tx: i as u32,
            amount: Some(dec!(1.0)),
        };

        let engine = engine.clone_handle();

        let handle = tokio::spawn(async move {
            engine.process_transaction(tx).await.unwrap();
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let elapsed = start.elapsed();
    let throughput = num_transactions as f64 / elapsed.as_secs_f64();

    println!(
        "Processed {} transactions in {:?} ({:.0} tx/sec)",
        num_transactions, elapsed, throughput
    );

    // Should be very fast (tens of thousands per second)
    assert!(throughput > 1000.0, "Throughput too low: {}", throughput);
}
