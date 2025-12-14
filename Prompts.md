# Initial Prompts
I used Claude Code as a development companion during the time I developed this project. I used an initial set of prompts to incredemtally build the project's functional and not-functional requirements, design, data model, edge cases handling, etc. 

This Readme includes the important prompts, and not meant to include an exhaustive list.
---

## Prompt 1: Project Setup & Data Models

> Create a new Rust project called `payments-engine` with the following dependencies: serde (with derive), csv, rust_decimal (with serde-with-str feature), and thiserror.
>
> Define the core data models:
> - `TransactionType` enum: Deposit, Withdrawal, Dispute, Resolve, Chargeback
> - `Transaction` struct for CSV input: type, client (u16), tx (u32), amount (Option<Decimal>)
> - `Account` struct: client_id, available, held, locked (total is derived)
> - `StoredTransaction` struct: tx_id, client_id, amount, disputed flag
>
> Use serde for CSV deserialization. Handle the lowercase transaction type names and optional amount field.

---

## Prompt 2: CSV Reader (Streaming)

> Implement a streaming CSV reader that:
> - Takes a file path as input
> - Returns an iterator over `Result<Transaction, Error>`
> - Handles whitespace trimming in fields
> - Skips malformed rows gracefully (log to stderr, continue processing)
> - Uses the csv crate with serde deserialization

---

## Prompt 3: Engine Core - Deposit & Withdrawal

> Implement the `PaymentsEngine` struct with:
> - `HashMap<u16, Account>` for accounts
> - `HashMap<u32, StoredTransaction>` for transaction history
> - `process_transaction(&mut self, tx: Transaction)` method
>
> Implement deposit: add to available, store transaction for disputes
> Implement withdrawal: subtract from available if sufficient funds, ignore otherwise
> Skip all transactions if account is locked.

---

## Prompt 4: Engine Core - Dispute, Resolve, Chargeback

> Extend the `PaymentsEngine` to handle:
>
> **Dispute**: Look up tx by ID, verify client matches, move funds from available to held, mark as disputed. Ignore if tx doesn't exist, wrong client, or already disputed.
>
> **Resolve**: Look up disputed tx, move funds from held back to available, clear disputed flag. Ignore if not under dispute.
>
> **Chargeback**: Look up disputed tx, remove funds from held, lock the account. Ignore if not under dispute.

---

## Prompt 5: CSV Writer & Main

> Implement CSV output that:
> - Writes header: `client,available,held,total,locked`
> - Iterates over all accounts and writes each as a CSV row
> - Outputs to stdout (not a file)
> - Formats decimals with up to 4 decimal places
>
> Implement main():
> - Parse command line args to get input file path
> - Create engine, stream transactions, process each
> - Output final account states
> - Handle file-not-found with exit code 1

---

## Prompt 6: Error Handling & Edge Cases

> Add robust error handling:
> - Custom error type using thiserror
> - File errors → stderr message + exit code 1
> - Parse errors → log to stderr, skip row, continue
> - Business logic errors → silently ignore (per spec)
>
> Handle edge cases:
> - Duplicate transaction IDs (ignore second)
> - Negative amounts (ignore)
> - Client mismatch on dispute/resolve/chargeback (ignore)

---

## Prompt 7: Unit & Integration Tests

> Write tests for:
> 1. Transaction parsing (all types, whitespace, precision variations)
> 2. Deposit/withdrawal logic (including insufficient funds)
> 3. Full dispute→resolve flow
> 4. Full dispute→chargeback flow (verify account locks)
> 5. Locked account ignores new transactions
> 6. Client ID mismatch on dispute is rejected
>
> Create test fixture CSVs and verify output matches expected.

---

## Prompt 8: README Documentation

> Write a README.md covering:
> - How to build and run (`cargo build --release`, `cargo run -- file.csv`)
> - Design decisions and assumptions made
> - How disputes/chargebacks work
> - Error handling approach
> - Testing approach (how to run tests)
> - Any limitations or future improvements




# Generic Testing Prompt
“Create a complete testing approach for my Rust CLI application.
Follow these guidelines:
## Unit Tests
    • Test pure logic, parsing, validation, formatting.
    • Include happy paths, error paths, and edge cases.
    • Prefer table-driven tests.
    • Use #[cfg(test)] modules close to the code.
## Integration Tests
    • Place tests in the tests/ directory.
    • Test full CLI behavior: flags, argument handling, file I/O, invalid inputs, exit codes.
    • Use temporary directories instead of real global state.
## Mocking & Stubbing
    • Identify external dependencies (file system, network, env vars).
    • Introduce traits to allow mocking those dependencies.
    • Use simple manual mocks/stubs; avoid unnecessary mocking frameworks.
## Edge Cases
    • Explicitly list critical edge cases for this app and create tests for them.
## Coverage
    • Focus on business logic, not argument-wiring boilerplate.
    • Recommend a tool (e.g., cargo tarpaulin) and show how to interpret coverage.
## Output
    • Provide:
        1. A brief testing strategy summary
        2. Example unit tests
        3. Example integration tests
        4. Suggested mocks/stubs

Apply idiomatic Rust testing practices used in real CLI tools.


# GEneric Error Handling Prompt
• Use Result<T, E> for all recoverable errors.
• Use thiserror to define typed error enums inside library or module code.
• Use anyhow::Result only at the top-level (main or CLI orchestration).
• Replace all .unwrap() and .expect() with ? unless the failure would be a programming error.
• Add error context using .context(…) where it improves diagnostics.
• Keep functions flat and readable by using the ? operator instead of nested matches.
• Only panic when encountering truly unrecoverable conditions or bugs.
• Apply the idiomatic Rust style used in real production CLIs.
After refactoring, explain:
– the error types you created and why
– how errors propagate
– where context is added and why
– any edge cases you handled