# Faucet Contract

This is a token faucet smart contract on Soroban. It dispenses test tokens to users while enforcing a claim limit and a cooldown period between claims.

## Public Functions

### `initialize(env: Env, admin: Address, token: Address, claim_limit: i128, cooldown: u64)`
Initializes the faucet contract with its settings.
* **Parameters**:
  * `env`: The execution environment.
  * `admin`: The administrator address of the faucet.
  * `token`: The address of the token to be dispensed.
  * `claim_limit`: The maximum amount of tokens allowed per claim.
  * `cooldown`: The cooldown period in seconds required between claims for a single user.
* **Returns**: None.

### `claim(env: Env, user: Address, amount: i128)`
Allows a user to claim tokens from the faucet, up to the `claim_limit`, and enforces the `cooldown` period.
* **Parameters**:
  * `env`: The execution environment.
  * `user`: The address claiming the tokens. Must have authorization.
  * `amount`: The amount of tokens requested.
* **Returns**: None.

---

# Wrapped Token Contract

A mint/burn wrapper that turns an existing Soroban/Stellar asset (e.g. USDC) into a Soroban-native token. Users deposit the underlying asset and receive wrapped tokens 1:1 via `wrap`, and can redeem them back at any time via `unwrap`. The admin can additionally `mint` wrapped tokens directly and any holder can `burn` their own tokens.

## Public Functions

### `initialize(env: Env, admin: Address, underlying: Address, name: String, symbol: String, decimals: u32)`
Initializes the wrapper contract. Must be called exactly once after deployment.
* **Parameters**:
  * `env`: The execution environment.
  * `admin`: Administrator address (permitted to call `mint`).
  * `underlying`: Address of the existing Soroban token being wrapped (e.g. USDC contract address).
  * `name`: Display name for the wrapped token (e.g. `"Wrapped USDC"`).
  * `symbol`: Short ticker symbol for the wrapped token (e.g. `"wUSDC"`).
  * `decimals`: Decimal precision — normally matches the underlying asset (7 for Stellar assets).
* **Returns**: None.

### `wrap(env: Env, caller: Address, amount: i128) -> i128`
Deposits `amount` of the underlying asset from `caller` into the contract and mints an equal amount of wrapped tokens to `caller` (1:1 peg). The caller must have authorized the transfer of the underlying asset.
* **Parameters**:
  * `env`: The execution environment.
  * `caller`: Address performing the wrap. Must authorize this call.
  * `amount`: Number of underlying tokens to deposit. Must be positive.
* **Returns**: The new wrapped-token balance of `caller` after wrapping.

### `unwrap(env: Env, caller: Address, amount: i128) -> i128`
Burns `amount` of wrapped tokens held by `caller` and returns an equal amount of the underlying asset (1:1 peg).
* **Parameters**:
  * `env`: The execution environment.
  * `caller`: Address performing the unwrap. Must authorize this call.
  * `amount`: Number of wrapped tokens to burn and redeem. Must be positive and ≤ caller's wrapped balance.
* **Returns**: The new wrapped-token balance of `caller` after unwrapping.

### `mint(env: Env, admin: Address, recipient: Address, amount: i128)`
Admin-only. Mints wrapped tokens directly to `recipient` without requiring a deposit of the underlying asset. Useful for incentive programs or testing.
* **Parameters**:
  * `env`: The execution environment.
  * `admin`: The contract administrator. Must authorize this call.
  * `recipient`: Address that receives the newly minted wrapped tokens.
  * `amount`: Number of wrapped tokens to mint. Must be positive.
* **Returns**: None.

### `burn(env: Env, caller: Address, amount: i128)`
Permanently destroys `amount` of wrapped tokens held by `caller`. No underlying asset is returned — use `unwrap` if you want to reclaim the underlying.
* **Parameters**:
  * `env`: The execution environment.
  * `caller`: Address whose wrapped tokens will be burned. Must authorize this call.
  * `amount`: Number of wrapped tokens to burn. Must be positive and ≤ caller's wrapped balance.
* **Returns**: None.

### `balance(env: Env, account: Address) -> i128`
Returns the wrapped-token balance of `account`.
* **Parameters**:
  * `env`: The execution environment.
  * `account`: The address to query.
* **Returns**: Balance as `i128` (0 for unknown accounts).

### `total_supply(env: Env) -> i128`
Returns the total number of wrapped tokens currently in circulation.
* **Parameters**:
  * `env`: The execution environment.
* **Returns**: Total supply as `i128`.

### `name(env: Env) -> String`
Returns the human-readable name of the wrapped token (set at initialization).
* **Parameters**:
  * `env`: The execution environment.
* **Returns**: Token name as a Soroban `String`.

### `symbol(env: Env) -> String`
Returns the ticker symbol of the wrapped token (set at initialization).
* **Parameters**:
  * `env`: The execution environment.
* **Returns**: Token symbol as a Soroban `String`.

### `decimals(env: Env) -> u32`
Returns the decimal precision of the wrapped token (set at initialization).
* **Parameters**:
  * `env`: The execution environment.
* **Returns**: Decimal places as `u32`.

### `underlying(env: Env) -> Address`
Returns the address of the underlying Soroban asset being wrapped.
* **Parameters**:
  * `env`: The execution environment.
* **Returns**: The underlying token contract address.
