# Design Decisions

## Concurrency
The requirement states: "What if your code was bundled in a server, and these CSVs came from thousands of concurrent TCP streams?" This presents two challenges: (1) **connection handling** - efficiently managing thousands of simultaneous connections without exhausting system resources, and (2) **throughput** - processing high transaction volumes while avoiding lock contention that would serialize all operations and create a bottleneck.

**Option A: Standard Library Threads** - Use `std::thread` with `Arc<Mutex<PaymentsEngine>>`. Simple and requires no external dependencies, but thread-per-connection doesn't scale beyond ~500 connections (each thread consumes ~2MB stack), and a single mutex serializes all transaction processing regardless of CPU core count, limiting throughput to single-core performance (~2K tx/sec).

**Option B: Tokio Async (No Sharding)** - Use tokio async runtime with `Arc<RwLock<PaymentsEngine>>`. Efficiently handles 10,000+ concurrent connections with lightweight tasks, but still has a single write lock that serializes all transaction processing, limiting throughput to ~2K tx/sec despite multiple cores being available.

**Option C: Tokio + Sharding** - Use tokio async runtime with multiple independent engine shards, partitioned by `client_id % num_shards`. Handles 10,000+ concurrent connections efficiently AND enables parallel transaction processing across CPU cores by routing different clients to different shards, each with its own independent lock. Throughput scales linearly with shard count (~16K tx/sec with 8 shards, ~30K+ with 16 shards).

**I chose Option C: Tokio + Sharding** because the requirement specifically mentions "thousands of concurrent TCP streams" which implies both high connection count and high transaction volume. Tokio solves the connection scalability problem (10,000+ connections efficiently), while sharding solves the throughput problem (parallel processing across cores). The implementation uses `Vec<Arc<RwLock<PersistentEngine<StubPersistence>>>>` where each client is consistently routed to the same shard via `client_id % num_shards`, ensuring same-client transactions remain serialized (for consistency) while different-client transactions can process in parallel (for throughput). Each shard integrates persistence support for crash recovery. This architecture demonstrates production-ready thinking for server deployment scenarios and directly addresses the efficiency requirement around handling large concurrent workloads.


## Support multi-account in the future
The requirements mention a client will have a single account, but for maintainability I considered the case of supporting multiple accounts per customer, in the future. There are different options for modeling this relationship:
1) A client entity that has all of the balance properties, and no account entity.
2) An account entity that holds properties (available, held, total, locked) this will have client_id as a property, or 
3) Create both account and client entities and support multi-account use case from now.
Option 1 will require more refactor to add the account struct, and map client_id to accounts.  
Option 2 (Chosen) since the account struct will not change, we will only need to create a client struct 
Option 3,  This will add complexity early on with no value since the input CSV doesn't have account_id

## Account 'total' field 
I considered two approaches for the account 'total' field: 
(1) storing it as a field and manually calling update_total() method after every balance change, or 
(2) removing the stored field entirely and computing it on-demand via a total() method that returns available + held. 
I chose the computed method approach because it follows the Single Source of Truth principle - available and held are the authoritative state. This eliminates the maintenance burden of remembering to call update_total() after every operation, reduces the risk of bugs from forgotten updates, and simplifies the codebase. 

Prompt -->  remove total attribute, update_total method and add total() method. make all of the code changes needed and make sure to update tests, run them, and make sure they pass.


## Account balance update
I considered three approaches to handle the account balance updates: 
(1) keeping separate methods for each operation (deposit, withdraw, hold, release, chargeback), 
(2) a single generic update_balance(available_delta, held_delta) function, or 
(3) combining similar operations like adjust_available(delta) for deposit/withdraw. 
I chose option (1) because financial systems prioritize explicit intent and safety over code conciseness. The key advantages are: business logic is self-documenting (account.deposit(100) vs update_balance(100, 0)), each method can have operation-specific validation (withdraw checks insufficient funds), and it's harder to introduce sign errors or parameter confusion. 


## Validation logic
I wanted to determine where to place validation logic in code. I considered three approaches: 
(1) all validation in the engine with account methods as simple setters, 
(2) all validation in account methods, or 
(3) layered validation split by concern. 
I chose the layered approach because it provides a clear separation of responsibilities - the account layer validates data integrity constraints that depend only on its own state (sufficient balance, account not locked), while the engine layer validates business logic that requires external  context (transaction exists, client ID matches). This approach follows encapsulation principles by keeping balance rules with the data they protect, provides defense in depth against bugs, makes the account reusable and testable in isolation, and creates maintainability by giving future developers clear places to look for different types of validation. The tradeoff is slightly more complex account methods that return booleans, but this is justified by the improved safety and architectural clarity.

## Decision inferred from the requirements
1. **Decimal Precision**: Uses `rust_decimal` to avoid floating-point errors
2. **Streaming**: Processes CSV line-by-line for memory efficiency
3. **Dispute Storage**: Only stores deposit transactions for dispute reference
4. **Security**: Validates client ID matches for all dispute-related operations
5. **Error Handling**: Never panics on bad input; silently ignores and continues
6. **Account Locking**: Once locked by chargeback, account rejects all transactions