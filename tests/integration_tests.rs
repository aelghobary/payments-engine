mod common;

use std::fs::File;

use common::{assert_client_balance, build_csv, process_csv_string};
use payments_engine::process_transactions;

#[test]
fn test_comprehensive_scenario() {
    // Tests a complex multi-client scenario from CSV file
    // - Client 1: Multiple deposits/withdrawals with dispute->resolve cycle
    // - Client 2: Simple deposits and withdrawals
    // - Client 3: Deposit with dispute->chargeback (account locked)
    let input = File::open("tests/fixtures/comprehensive_test.csv").unwrap();
    let mut output = Vec::new();

    process_transactions(input, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Comprehensive test output:\n{}", output_str);

    // Client 1: 1000 + 500 - 200 + 100 - 50 = 1350
    // (dispute on tx 3 (500) was resolved, so funds are available)
    assert!(output_str.contains("1,1350"));
    assert!(output_str.contains("1350.0,false"));

    // Client 2: 2000 - 300 = 1700
    assert!(output_str.contains("2,1700"));
    assert!(output_str.contains("1700.0,false"));

    // Client 3: 5000 deposited, disputed, then chargedback = 0
    // Tried to deposit 1000 after chargeback but account is locked
    assert!(output_str.contains("3,0"));
    assert!(output_str.contains("0.0,true"));
}

#[test]
fn test_basic_transactions() {
    let input = File::open("tests/fixtures/basic.csv").unwrap();
    let mut output = Vec::new();

    process_transactions(input, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Basic output:\n{}", output_str);

    // Client 1: 100 + 50 - 25 = 125
    assert!(output_str.contains("client,available,held,total,locked"));
    assert!(output_str.contains("1,125"));
    assert!(output_str.contains("125.0,false"));
}

#[test]
fn test_dispute_resolve_flow() {
    let input = File::open("tests/fixtures/disputes.csv").unwrap();
    let mut output = Vec::new();

    process_transactions(input, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Disputes output:\n{}", output_str);

    // Client 1: disputed and resolved, back to 100
    assert!(output_str.contains("1,100.0,0"));
    assert!(output_str.contains("100.0,false"));
    // Client 2: no disputes, 200
    assert!(output_str.contains("2,200"));
    assert!(output_str.contains("200.0,false"));
}

#[test]
fn test_chargeback_locks_account() {
    let input = File::open("tests/fixtures/chargebacks.csv").unwrap();
    let mut output = Vec::new();

    process_transactions(input, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Chargebacks output:\n{}", output_str);

    // Client 1: deposited 100 + 50, disputed 100, chargedback 100, tried to deposit 25 (ignored)
    // Final: available=50, held=0, total=50, locked=true
    assert!(output_str.contains("1,50"));
    assert!(output_str.contains("50.0,true"));
}

#[test]
fn test_edge_cases() {
    let input = File::open("tests/fixtures/edge_cases.csv").unwrap();
    let mut output = Vec::new();

    process_transactions(input, &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Edge cases output:\n{}", output_str);

    // Client 1: 100 (withdrawal of 150 fails), then +50.5 = 150.5
    assert!(output_str.contains("1,150.5"));

    // Client 2: 0.0001 deposited, withdrawal of 0.00005
    assert!(output_str.contains("2,0.00005"));
}

#[test]
fn test_multiple_clients() {
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,2,2,200.0
deposit,3,3,300.0
withdrawal,1,4,50.0
dispute,2,2,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Multiple clients output:\n{}", output_str);

    // Should have 3 clients
    assert!(output_str.contains("1,50"));
    assert!(output_str.contains("2,0"));
    assert!(output_str.contains("200.0,200.0"));
    assert!(output_str.contains("3,300"));
}

#[test]
fn test_dispute_with_insufficient_available() {
    // Test that dispute fails when there's insufficient available balance
    let input = "type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,80.0
dispute,1,1,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!(
        "Dispute with insufficient available output:\n{}",
        output_str
    );

    // Client 1: deposited 100, withdrew 80, dispute should fail (only 20 available, need 100)
    // Result: available=20, held=0, total=20
    assert!(output_str.contains("1,20"));
    assert!(output_str.contains("20.0,0"));
    assert!(output_str.contains("20.0,false"));
}

#[test]
fn test_locked_account_rejects_withdrawal() {
    // Test that locked account rejects withdrawals
    let input = "type,client,tx,amount
deposit,1,1,200.0
dispute,1,1,
chargeback,1,1,
deposit,1,2,100.0
withdrawal,1,3,50.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Locked account rejects withdrawal output:\n{}", output_str);

    // Account is locked after chargeback, both deposit and withdrawal should be ignored
    assert!(output_str.contains("1,0.0,0.0,0.0,true"));
}

#[test]
fn test_dispute_then_withdrawal_reduces_available() {
    // Test that disputed funds are held, so withdrawal can't touch them
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
dispute,1,1,
withdrawal,1,3,60.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Dispute then withdrawal output:\n{}", output_str);

    // Client 1: deposited 100+50=150, disputed 100 (moves to held), available=50
    // Withdrawal of 60 should fail (only 50 available)
    // Result: available=50, held=100, total=150
    assert!(output_str.contains("1,50"));
    assert!(output_str.contains("100.0,150"));
}

#[test]
fn test_duplicate_withdrawal_rejected() {
    // Test that duplicate withdrawal transaction IDs are rejected
    let input = "type,client,tx,amount
deposit,1,1,200.0
withdrawal,1,2,50.0
withdrawal,1,2,50.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Duplicate withdrawal output:\n{}", output_str);

    // Second withdrawal with same ID should be ignored
    // Result: 200 - 50 = 150
    assert!(output_str.contains("1,150"));
    assert!(output_str.contains("150.0,false"));
}

#[test]
fn test_duplicate_deposit_rejected() {
    // Test that duplicate deposit transaction IDs are rejected
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,1,100.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Duplicate deposit output:\n{}", output_str);

    // Second deposit with same ID should be ignored
    // Result: only 100, not 200
    assert!(output_str.contains("1,100"));
    assert!(output_str.contains("100.0,false"));
}

#[test]
fn test_redispute_after_resolve() {
    // Test that a transaction can be disputed again after being resolved
    let input = "type,client,tx,amount
deposit,1,5,100.0
dispute,1,5,
resolve,1,5,
dispute,1,5,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Re-dispute output:\n{}", output_str);

    // After dispute → resolve → dispute: funds should be held again
    // Result: available=0, held=100, total=100
    assert!(output_str.contains("1,0"));
    assert!(output_str.contains("100.0,100"));
}

#[test]
fn test_cross_client_dispute_rejected() {
    // Test security: Client 2 cannot dispute Client 1's transaction
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,2,2,200.0
dispute,2,1,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Cross-client dispute output:\n{}", output_str);

    // Client 2 trying to dispute Client 1's tx should be ignored
    // Client 1: 100 available (not disputed)
    // Client 2: 200 available (not involved in dispute)
    assert!(output_str.contains("1,100"));
    assert!(output_str.contains("2,200"));
}

#[test]
fn test_resolve_without_dispute() {
    // Test that resolve on non-disputed transaction is ignored
    let input = "type,client,tx,amount
deposit,1,1,100.0
resolve,1,1,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Resolve without dispute output:\n{}", output_str);

    // Resolve should be ignored, funds remain available
    assert!(output_str.contains("1,100"));
    assert!(output_str.contains("0,100"));
}

#[test]
fn test_chargeback_without_dispute() {
    // Test that chargeback on non-disputed transaction is ignored
    let input = "type,client,tx,amount
deposit,1,1,100.0
chargeback,1,1,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Chargeback without dispute output:\n{}", output_str);

    // Chargeback should be ignored, account not locked
    assert!(output_str.contains("1,100"));
    assert!(output_str.contains("false"));
}

#[test]
fn test_dispute_nonexistent_transaction() {
    // Test that disputing a non-existent transaction is ignored
    let input = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,999,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Dispute nonexistent tx output:\n{}", output_str);

    // Dispute on tx 999 (doesn't exist) should be ignored
    assert!(output_str.contains("1,100"));
    assert!(output_str.contains("0,100"));
}

#[test]
fn test_negative_amount_rejected() {
    // Test that negative amounts are rejected
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,-50.0
withdrawal,1,3,-25.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Negative amount output:\n{}", output_str);

    // Only first deposit should succeed
    assert!(output_str.contains("1,100"));
}

#[test]
fn test_zero_amount_rejected() {
    // Test that zero amounts are rejected
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,0.0
withdrawal,1,3,0.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Zero amount output:\n{}", output_str);

    // Only first deposit should succeed
    assert!(output_str.contains("1,100"));
}

#[test]
fn test_full_account_lifecycle() {
    // Test complete lifecycle: deposit → dispute → resolve → withdrawal
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
dispute,1,1,
resolve,1,1,
withdrawal,1,3,75.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Full lifecycle output:\n{}", output_str);

    // After all operations: 100 + 50 - 75 = 75
    assert!(output_str.contains("1,75"));
    assert!(output_str.contains("0,75"));
}

#[test]
fn test_dispute_after_chargeback() {
    // Test that a transaction cannot be disputed after being charged back
    let input = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
chargeback,1,1,
dispute,1,1,
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Dispute after chargeback output:\n{}", output_str);

    // After chargeback, second dispute should be ignored
    // Account is locked with 0 balance
    assert!(output_str.contains("1,0"));
    assert!(output_str.contains("true"));
}

#[test]
fn test_complex_multi_client_scenario() {
    // Test complex scenario with multiple clients and interleaved operations
    let input = "type,client,tx,amount
deposit,1,1,1000.0
deposit,2,2,500.0
deposit,3,3,750.0
withdrawal,1,4,100.0
dispute,2,2,
deposit,1,5,200.0
withdrawal,3,6,50.0
resolve,2,2,
chargeback,3,3,
deposit,3,7,100.0
";
    let mut output = Vec::new();

    process_transactions(input.as_bytes(), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Complex multi-client output:\n{}", output_str);

    // Client 1: 1000 - 100 + 200 = 1100
    assert!(output_str.contains("1,1100"));

    // Client 2: 500, disputed then resolved = 500
    assert!(output_str.contains("2,500"));

    // Client 3: 750 deposited, 50 withdrawn = 700
    // Chargeback on tx 3 without dispute → ignored (not under dispute)
    // Deposit 100 → 700 + 100 = 800
    assert!(output_str.contains("3,800"));
}

// =============================================================================
// TABLE-DRIVEN INTEGRATION TESTS
// =============================================================================
// These tests use the table-driven pattern to efficiently test multiple
// variations of similar scenarios. Each test contains a vector of test cases
// with different inputs and expected outcomes, reducing code duplication.

/// Table-driven test for invalid amount scenarios (negative, zero)
#[test]
fn test_invalid_amounts_table_driven() {
    struct TestCase {
        name: &'static str,
        transactions: &'static str,
        should_have_account: bool,
        expected_balance: Option<&'static str>,
    }

    let test_cases = vec![
        TestCase {
            name: "negative deposit",
            transactions: "deposit,1,1,-100.0\n",
            should_have_account: false,
            expected_balance: None,
        },
        TestCase {
            name: "zero deposit",
            transactions: "deposit,1,1,0.0\n",
            should_have_account: false,
            expected_balance: None,
        },
        TestCase {
            name: "negative withdrawal",
            transactions: "deposit,1,1,100.0\nwithdrawal,1,2,-50.0\n",
            should_have_account: true,
            expected_balance: Some("100"),
        },
        TestCase {
            name: "zero withdrawal",
            transactions: "deposit,1,1,100.0\nwithdrawal,1,3,0.0\n",
            should_have_account: true,
            expected_balance: Some("100"),
        },
    ];

    for case in test_cases {
        let csv = format!("type,client,tx,amount\n{}", case.transactions);
        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        if case.should_have_account {
            let balance = case.expected_balance.unwrap();
            assert_client_balance(&output, 1, balance, "0", balance, false);
        } else {
            assert!(
                !output.contains("1,"),
                "Test '{}' failed: Client should not exist.\nOutput:\n{}",
                case.name,
                output
            );
        }
    }
}

/// Table-driven test for insufficient funds scenarios
#[test]
fn test_insufficient_funds_table_driven() {
    struct TestCase {
        name: &'static str,
        deposit: &'static str,
        withdrawal: &'static str,
        expected_balance: &'static str,
    }

    let test_cases = vec![
        TestCase {
            name: "withdraw more than balance",
            deposit: "100.0",
            withdrawal: "150.0",
            expected_balance: "100",
        },
        TestCase {
            name: "withdraw exact balance plus one cent",
            deposit: "100.0",
            withdrawal: "100.01",
            expected_balance: "100",
        },
        TestCase {
            name: "small balance large withdrawal",
            deposit: "0.01",
            withdrawal: "100.0",
            expected_balance: "0.01",
        },
    ];

    for case in test_cases {
        let csv = build_csv(&[
            ("deposit", 1, 1, case.deposit),
            ("withdrawal", 1, 2, case.withdrawal),
        ]);

        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        assert_client_balance(
            &output,
            1,
            case.expected_balance,
            "0",
            case.expected_balance,
            false,
        );
    }
}

/// Table-driven test for dispute workflow variations
#[test]
fn test_dispute_workflows_table_driven() {
    struct TestCase {
        name: &'static str,
        transactions: Vec<(&'static str, u16, u32, &'static str)>,
        expected_available: &'static str,
        expected_held: &'static str,
        expected_locked: bool,
    }

    let test_cases = vec![
        TestCase {
            name: "dispute and resolve",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("dispute", 1, 1, ""),
                ("resolve", 1, 1, ""),
            ],
            expected_available: "100",
            expected_held: "0",
            expected_locked: false,
        },
        TestCase {
            name: "dispute and chargeback",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("dispute", 1, 1, ""),
                ("chargeback", 1, 1, ""),
            ],
            expected_available: "0",
            expected_held: "0",
            expected_locked: true,
        },
        TestCase {
            name: "dispute only",
            transactions: vec![("deposit", 1, 1, "100.0"), ("dispute", 1, 1, "")],
            expected_available: "0",
            expected_held: "100",
            expected_locked: false,
        },
        TestCase {
            name: "dispute resolve dispute again",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("dispute", 1, 1, ""),
                ("resolve", 1, 1, ""),
                ("dispute", 1, 1, ""),
            ],
            expected_available: "0",
            expected_held: "100",
            expected_locked: false,
        },
    ];

    for case in test_cases {
        let csv = build_csv(&case.transactions);
        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        let expected_total = format!(
            "{}",
            case.expected_available.parse::<f64>().unwrap()
                + case.expected_held.parse::<f64>().unwrap()
        );

        assert_client_balance(
            &output,
            1,
            case.expected_available,
            case.expected_held,
            &expected_total,
            case.expected_locked,
        );
    }
}

/// Table-driven test for decimal precision handling
#[test]
fn test_precision_scenarios_table_driven() {
    struct TestCase {
        name: &'static str,
        amount1: &'static str,
        amount2: &'static str,
        expected_total: &'static str,
    }

    let test_cases = vec![
        TestCase {
            name: "four decimal places",
            amount1: "0.0001",
            amount2: "0.0002",
            expected_total: "0.0003",
        },
        TestCase {
            name: "whole numbers",
            amount1: "100",
            amount2: "200",
            expected_total: "300",
        },
        TestCase {
            name: "mixed precision",
            amount1: "100.5",
            amount2: "50.25",
            expected_total: "150.75",
        },
        TestCase {
            name: "very small amounts",
            amount1: "0.0001",
            amount2: "0.0001",
            expected_total: "0.0002",
        },
    ];

    for case in test_cases {
        let csv = build_csv(&[
            ("deposit", 1, 1, case.amount1),
            ("deposit", 1, 2, case.amount2),
        ]);

        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        assert!(
            output.contains(&format!("1,{}", case.expected_total)),
            "Test '{}' failed. Expected total: {}\nOutput:\n{}",
            case.name,
            case.expected_total,
            output
        );
    }
}

/// Table-driven test for locked account behavior
#[test]
fn test_locked_account_operations_table_driven() {
    struct TestCase {
        name: &'static str,
        operation_after_lock: (&'static str, &'static str),
        expected_balance: &'static str,
    }

    // Base scenario: deposit 200, dispute it, chargeback it (account locked with 0 balance)
    let test_cases = vec![
        TestCase {
            name: "deposit after lock",
            operation_after_lock: ("deposit", "100.0"),
            expected_balance: "0",
        },
        TestCase {
            name: "withdrawal after lock",
            operation_after_lock: ("withdrawal", "50.0"),
            expected_balance: "0",
        },
    ];

    for case in test_cases {
        let mut transactions = vec![
            ("deposit", 1, 1, "200.0"),
            ("dispute", 1, 1, ""),
            ("chargeback", 1, 1, ""),
        ];

        // Add the operation after lock
        let (op_type, amount) = case.operation_after_lock;
        transactions.push((op_type, 1, 2, amount));

        let csv = build_csv(&transactions);
        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        assert_client_balance(
            &output,
            1,
            case.expected_balance,
            "0",
            case.expected_balance,
            true,
        );
    }
}

/// Table-driven test for multi-client isolation
#[test]
fn test_multi_client_isolation_table_driven() {
    struct ClientExpectation {
        client_id: u16,
        available: &'static str,
        held: &'static str,
    }

    struct TestCase {
        name: &'static str,
        transactions: Vec<(&'static str, u16, u32, &'static str)>,
        expectations: Vec<ClientExpectation>,
    }

    let test_cases = vec![
        TestCase {
            name: "three independent clients",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("deposit", 2, 2, "200.0"),
                ("deposit", 3, 3, "300.0"),
            ],
            expectations: vec![
                ClientExpectation {
                    client_id: 1,
                    available: "100",
                    held: "0",
                },
                ClientExpectation {
                    client_id: 2,
                    available: "200",
                    held: "0",
                },
                ClientExpectation {
                    client_id: 3,
                    available: "300",
                    held: "0",
                },
            ],
        },
        TestCase {
            name: "client 1 disputed, client 2 normal",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("deposit", 2, 2, "200.0"),
                ("dispute", 1, 1, ""),
            ],
            expectations: vec![
                ClientExpectation {
                    client_id: 1,
                    available: "0",
                    held: "100",
                },
                ClientExpectation {
                    client_id: 2,
                    available: "200",
                    held: "0",
                },
            ],
        },
        TestCase {
            name: "interleaved operations",
            transactions: vec![
                ("deposit", 1, 1, "100.0"),
                ("deposit", 2, 2, "200.0"),
                ("withdrawal", 1, 3, "25.0"),
                ("deposit", 2, 4, "50.0"),
                ("withdrawal", 2, 5, "100.0"),
            ],
            expectations: vec![
                ClientExpectation {
                    client_id: 1,
                    available: "75",
                    held: "0",
                },
                ClientExpectation {
                    client_id: 2,
                    available: "150",
                    held: "0",
                },
            ],
        },
    ];

    for case in test_cases {
        let csv = build_csv(&case.transactions);
        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        for expectation in case.expectations {
            let total = format!(
                "{}",
                expectation.available.parse::<f64>().unwrap()
                    + expectation.held.parse::<f64>().unwrap()
            );

            assert_client_balance(
                &output,
                expectation.client_id,
                expectation.available,
                expectation.held,
                &total,
                false,
            );
        }
    }
}

/// Table-driven test for duplicate transaction detection
#[test]
fn test_duplicate_detection_table_driven() {
    struct TestCase {
        name: &'static str,
        transactions: Vec<(&'static str, u16, u32, &'static str)>,
        expected_balance: &'static str,
    }

    let test_cases = vec![
        TestCase {
            name: "duplicate deposit same amount",
            transactions: vec![("deposit", 1, 1, "100.0"), ("deposit", 1, 1, "100.0")],
            expected_balance: "100", // Only first should process
        },
        TestCase {
            name: "duplicate deposit different amount",
            transactions: vec![("deposit", 1, 1, "100.0"), ("deposit", 1, 1, "50.0")],
            expected_balance: "100", // Only first should process
        },
        TestCase {
            name: "duplicate withdrawal",
            transactions: vec![
                ("deposit", 1, 1, "200.0"),
                ("withdrawal", 1, 2, "50.0"),
                ("withdrawal", 1, 2, "50.0"),
            ],
            expected_balance: "150", // 200 - 50 (second withdrawal ignored)
        },
        TestCase {
            name: "deposit and withdrawal same ID different clients",
            transactions: vec![("deposit", 1, 100, "100.0"), ("deposit", 2, 100, "200.0")],
            expected_balance: "100", // Client 1 should have 100 (tx IDs are global)
        },
    ];

    for case in test_cases {
        let csv = build_csv(&case.transactions);
        let output = process_csv_string(&csv).expect(&format!("Failed test: {}", case.name));

        assert_client_balance(
            &output,
            1,
            case.expected_balance,
            "0",
            case.expected_balance,
            false,
        );
    }
}
