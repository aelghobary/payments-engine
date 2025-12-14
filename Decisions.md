# Design Decisions

## Support multi-account in the future
The requirements mention a client will have a single account, but for maintainability I considered the case of supporting multiple accounts per customer, in the future. There are different options for modeling this relationship:
1) A client entity that has all of the balance properties, and no account entity.
2) An account entity that holds properties (available, held, total, cocked) this will have client_id as a proeprty, or 
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
5. **Error Handling**: Never panics on bad input; logs warnings and continues
6. **Account Locking**: Once locked by chargeback, account rejects all transactions