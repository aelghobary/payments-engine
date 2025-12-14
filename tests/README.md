# Test Organization Guide

This document explains the test organization structure and best practices for this project.

## Test Structure

```
tests/
├── common/
│   └── mod.rs              # Shared test helpers and utilities
├── fixtures/
│   ├── basic.csv           # Basic deposit/withdrawal scenarios
│   ├── disputes.csv        # Dispute and resolve workflows
│   ├── chargebacks.csv     # Chargeback scenarios
│   ├── edge_cases.csv      # Edge cases (precision, insufficient funds)
│   └── comprehensive_test.csv  # Complex multi-client scenario
├── concurrent_tests.rs     # Concurrency and sharding tests
├── integration_tests.rs    # End-to-end CSV processing tests (includes table-driven tests)
├── unit_account_tests.rs   # Unit tests for Account methods
└── unit_engine_tests.rs    # Unit tests for PaymentsEngine logic
```

## Test Categories

### 1. Unit Tests - Account (20 tests)
**File:** `unit_account_tests.rs`

Tests individual account methods in isolation:
- Account initialization
- Deposit/withdrawal operations
- Hold/release mechanisms
- Chargeback handling
- Precision handling (4 decimal places)
- Locked account behavior

**Philosophy:** Test the account data integrity layer without external dependencies.

### 2. Unit Tests - PaymentsEngine (21 tests)
**File:** `unit_engine_tests.rs`

Tests business logic and validation rules:
- Duplicate transaction detection
- Amount validation (negative, zero, missing)
- Transaction state validation
- Dispute workflow rules
- Client isolation
- Transaction ID uniqueness

**Philosophy:** Test business rule enforcement and engine logic.

### 3. Integration Tests (28 tests)
**File:** `integration_tests.rs`

End-to-end tests from CSV input to output, including both traditional and table-driven patterns:

**Traditional Integration Tests (21 tests):**
- 5 tests using CSV fixtures (complex scenarios)
- 16 tests using inline CSV (focused edge cases)

**Table-Driven Integration Tests (7 tests):**
Each test runs multiple scenarios efficiently using the table-driven pattern:
- Invalid amounts → 4 scenarios
- Insufficient funds → 3 scenarios
- Dispute workflows → 4 scenarios
- Precision handling → 4 scenarios
- Locked account operations → 2 scenarios
- Multi-client isolation → 3 scenarios
- Duplicate detection → 4 scenarios

**Table-driven tests** run multiple test cases efficiently through shared logic, reducing duplication while maintaining comprehensive coverage.

**Philosophy:** Test the complete processing pipeline. Use table-driven pattern for efficiently testing variations of similar scenarios.

### 4. Concurrency Tests (7 tests)
**File:** `concurrent_tests.rs`

Tests the ShardedEngine for high-concurrency scenarios:
- Concurrent deposits to same client
- Concurrent deposits to different clients
- Mixed concurrent operations (deposits, withdrawals, disputes)
- Concurrent dispute workflows
- High concurrency stress testing (1000+ transactions)
- Transaction ordering per client
- Throughput benchmarking

**Philosophy:** Verify the concurrent/sharded engine handles thousands of simultaneous transactions correctly without race conditions.

### 5. Common Test Helpers
**File:** `common/mod.rs`

Reusable utilities to reduce duplication:
- Transaction builders (`make_deposit`, `make_dispute`)
- CSV processing helpers (`process_csv_string`, `build_csv`)
- Custom assertions (`assert_client_balance`)

## When to Use CSV Fixtures vs Inline CSV

### Use CSV Fixture Files When:
✅ The CSV data is complex or lengthy (>10 lines)
✅ The same data might be reused across tests
✅ It represents a realistic production scenario
✅ You want to easily inspect/edit test data externally

**Example:**
```rust
#[test]
fn test_comprehensive_scenario() {
    let input = File::open("tests/fixtures/comprehensive_test.csv").unwrap();
    // ... test logic
}
```

### Use Inline CSV When:
✅ The test is short and self-contained (<10 lines)
✅ Testing a specific edge case or scenario
✅ The inline format makes the test more readable
✅ The test name clearly describes what's tested

**Example:**
```rust
#[test]
fn test_duplicate_deposit_rejected() {
    let input = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,1,100.0
";
    // ... test logic
}
```

## Current Test Coverage

**Total Tests:** 90
- Integration tests: 28
  - Traditional: 21 (5 with CSV fixtures, 16 inline)
  - Table-driven: 7 (testing 24 total scenarios)
- Unit account tests: 20
- Unit engine tests: 21
- Concurrency tests: 7
- Documentation tests: 14

## Understanding Table-Driven Tests

**Table-driven testing is a pattern, not a separate test category.** It's a way to structure tests that makes testing multiple variations of the same scenario more efficient.

**Key Benefit:** Write the test logic once, then specify multiple test cases as data.

**Example:**
```rust
// Instead of writing 4 separate test functions...
#[test]
fn test_withdraw_more_than_balance() { /* ... */ }
#[test]
fn test_withdraw_exact_balance_plus_one() { /* ... */ }
#[test]
fn test_small_balance_large_withdrawal() { /* ... */ }

// ...write ONE test with a table of cases:
#[test]
fn test_insufficient_funds_table_driven() {
    let test_cases = vec![
        TestCase { name: "withdraw more than balance", ... },
        TestCase { name: "withdraw exact balance plus one", ... },
        TestCase { name: "small balance large withdrawal", ... },
    ];
    for case in test_cases {
        // Test logic runs once for each case
    }
}
```

**When to use table-driven pattern:**
- Testing variations of the same logic with different inputs
- Multiple edge cases for the same feature
- Regression test suites that grow over time
- When you find yourself copy-pasting test code

## Adding New Tests

### For New Features
1. Add unit tests for new methods/functions
2. Add integration test for the end-to-end workflow
3. Consider table-driven tests if there are multiple similar scenarios

### For Bug Fixes
1. Add a failing test that reproduces the bug
2. Fix the bug
3. Verify the test passes
4. Consider if similar edge cases need coverage

### For CSV Fixtures
1. Place CSV file in `tests/fixtures/`
2. Name it descriptively (e.g., `withdrawal_edge_cases.csv`)
3. Add a corresponding test in `integration_tests.rs`
4. Document expected outcomes in test comments

## Best Practices

1. **Clear Test Names:** Use descriptive names like `test_dispute_with_insufficient_available`

2. **Arrange-Act-Assert:** Structure tests clearly:
   ```rust
   // Arrange
   let mut engine = PaymentsEngine::new();

   // Act
   engine.process_transaction(tx);

   // Assert
   assert_eq!(account.available, expected);
   ```

3. **Test One Thing:** Each test should verify one specific behavior

4. **Use Helpers:** Leverage common module helpers to reduce duplication

5. **Document Expected Behavior:** Add comments explaining calculations and expectations

6. **Print Debug Output:** Use `println!()` for debugging test failures

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_tests

# Run specific test
cargo test test_comprehensive_scenario

# Run with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests serially
cargo test -- --test-threads=1
```

## Test Maintenance

- Keep CSV fixtures small and focused
- Update tests when business rules change
- Remove obsolete tests
- Refactor common patterns into helpers
- Ensure all tests are documented
