# AMM — Automated Market Maker on Solana

Constant-product AMM (x·y=k) built with Anchor on Solana. Supports token swaps, liquidity provision, and withdrawal via LP tokens.

## Program ID

```
4Js6nztYixSyWu5tR7yta5Dr1xRbPYzZuwwZkqfDhatR
```

## Architecture

```
programs/amm/src/
├── lib.rs              # Program entrypoint, instruction dispatch
├── state.rs            # Config account (pool state)
├── constants.rs        # Program-wide constants
├── error.rs            # Custom error types
└── instructions/
    ├── initialize.rs   # Create pool config + LP mint
    ├── deposit.rs      # Add liquidity, receive LP tokens
    ├── withdraw.rs     # Burn LP tokens, receive token pair
    └── swap.rs         # Swap token X for Y or Y for X
```

## Instructions

### `initialize(seed, fee, authority)`
Creates a new liquidity pool for a token pair.
- `seed` — unique u64 to derive config PDA, allows multiple pools per pair
- `fee` — swap fee in basis points (e.g. `30` = 0.3%)
- `authority` — optional pubkey that can lock/unlock the pool

### `deposit(amount, max_x, max_y)`
Adds liquidity to the pool and mints LP tokens to the user.
- `amount` — LP tokens to mint
- `max_x` / `max_y` — slippage guards on token amounts deposited

### `withdraw(amount, min_x, min_y)`
Burns LP tokens and returns the proportional share of the pool.
- `amount` — LP tokens to burn
- `min_x` / `min_y` — slippage guards on tokens received

### `swap(is_x, amount_in, min_amount_out)`
Swaps one token for the other using the constant-product formula.
- `is_x` — `true` to swap X→Y, `false` to swap Y→X
- `amount_in` — tokens to send
- `min_amount_out` — minimum tokens to receive (slippage guard)

## State

```rust
pub struct Config {
    pub seed: u64,
    pub authority: Option<Pubkey>,  // pool lock authority
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16,                   // basis points
    pub locked: bool,
    pub config_bump: u8,
    pub lp_bump: u8,
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `anchor-lang 1.0.1` | Solana program framework |
| `anchor-spl 1.0.1` | SPL token CPI helpers |
| `constant-product-curve` | AMM math (x·y=k) |
| `litesvm 0.10.0` | Fast in-process SVM for tests |
| `litesvm-token 0.10.0` | Token helpers for LiteSVM tests |

## Testing

Tests run entirely in-process via LiteSVM — no local validator needed.

```
cargo test
```

```
Running unittests src/lib.rs (target/debug/deps/amm-ad4fd2f15732e581)

running 1 test
test test_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/tests.rs (target/debug/deps/tests-a2c4bf6918b521b2)

running 4 tests
test test_initialize ... ok
test test_deposit ... ok
test test_swap ... ok
test test_withdraw ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

   Doc-tests amm

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Build

```
anchor build
```

## Deploy

```
anchor deploy --provider.cluster devnet
```
