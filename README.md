# Payments Engine

A robust transaction processing engine that reads financial transactions from CSV, maintains client account states (including dispute handling), and outputs final account balances.

## Features

- Streaming CSV processing for memory efficiency
- Support for deposits, withdrawals, disputes, resolves, and chargebacks
- Precise decimal arithmetic using `rust_decimal` (4 decimal places)
- Comprehensive error handling with graceful failures
- Account locking on chargebacks
- Client mismatch protection for disputes
- Full test coverage (unit and integration tests)

## Usage

```bash
cargo run -- <input.csv> > accounts.csv
```

### Input Format

CSV file with the following columns:
- `type`: Transaction type (deposit, withdrawal, dispute, resolve, chargeback)
- `client`: Client ID (u16)
- `tx`: Transaction ID (u32)
- `amount`: Transaction amount (optional for dispute/resolve/chargeback)

Example:
```csv
type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
withdrawal,1,3,25.0
dispute,1,2,
resolve,1,2,
```

### Output Format

CSV to stdout with columns:
- `client`: Client ID
- `available`: Available funds
- `held`: Held funds (under dispute)
- `total`: Total funds (available + held)
- `locked`: Account locked status

Example:
```csv
client,available,held,total,locked
1,125.0,0.0,125.0,false
```

## Transaction Processing Rules

### Deposit
- Increases available balance
- Stored for potential disputes
- Ignored if account is locked

### Withdrawal
- Decreases available balance
- Fails silently if insufficient funds
- Ignored if account is locked

### Dispute
- Moves funds from available to held
- Only works on existing transactions
- Requires client ID match
- Ignored if already disputed

### Resolve
- Moves funds from held back to available
- Only works on disputed transactions
- Requires client ID match

### Chargeback
- Removes held funds permanently
- Locks the account
- Only works on disputed transactions
- Requires client ID match
- Account ignores all future transactions

## Testing

Run all tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test test_dispute_resolve
```

## Project Structure

```
payments-engine/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Public API & CSV processing
│   ├── engine.rs            # Transaction processing logic
│   ├── error.rs             # Error types
│   └── models/
│       ├── transaction.rs   # Input transaction types
│       ├── account.rs       # Client account state
│       └── stored_tx.rs     # Stored transaction for disputes
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/            # Test CSV files
└── Cargo.toml
```