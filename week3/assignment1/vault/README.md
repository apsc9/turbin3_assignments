# Vault Program

A Solana vault program built with Anchor, written from scratch as part of Turbin3 Week 3 Assignment.

## Overview

The vault program allows users to deposit and withdraw SOL through a PDA-controlled vault account. Each user gets their own vault derived from their public key.

## Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize` | Creates a vault state PDA and vault PDA for the user. Stores bump seeds for future CPI signing. |
| `deposit` | Transfers SOL from the user into the vault PDA via system program CPI. |
| `withdraw` | Transfers SOL from the vault PDA back to the user using PDA signer seeds. |
| `close` | Drains remaining SOL from the vault and closes the vault state account, reclaiming rent. |

## Account Architecture

- **VaultState** (`seeds: ["state", user]`) - Program-owned account storing bump seeds (vault_bump, state_bump).
- **Vault** (`seeds: ["vault", vault_state]`) - SystemAccount PDA that holds deposited SOL. Program signs for it via stored bumps.

## Tech Stack

- Anchor v1.0.0
- Rust
- LiteSVM for testing

## Build

```bash
anchor build
```

## Test

Tests are written in Rust using LiteSVM. A single integration test covers the full lifecycle: initialize, deposit, withdraw, and close.

```bash
cargo test
```

### Test Output

```
Running test suite: "/Users/apsc9/stack/turbin3/week3/assignment1/vault/Anchor.toml"

   Compiling vault v0.1.0 (/Users/apsc9/stack/turbin3/week3/assignment1/vault/programs/vault)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.74s
     Running unittests src/lib.rs (target/debug/deps/vault-e07013c68c4a9c0a)

running 1 test
test test_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/test_vault.rs (target/debug/deps/test_vault-9e0ac06aedf03717)

running 1 test
test test_initialize_deposit_withdraw_close ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests vault

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
