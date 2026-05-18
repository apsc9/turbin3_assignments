# Anchor Escrow Program

A token escrow program built on Solana using the Anchor framework. Enables trustless peer-to-peer token swaps — a maker deposits token A and specifies how much of token B they want in return. A taker can fulfill the swap, or the maker can reclaim their tokens.

## Program ID

```
5MEny7vyHCFMGSrLxKKZPUsaRZcR7KY5tvZoBu3b5hTE
```

## How It Works

The escrow holds three instructions:

| Instruction | Description |
|-------------|-------------|
| **Make** | Maker deposits token A into a PDA-owned vault and creates an escrow account storing the swap terms (desired amount of token B). |
| **Take** | Taker sends the requested token B to the maker, receives token A from the vault, and the escrow + vault are closed. |
| **Refund** | Maker reclaims their deposited token A from the vault. Escrow + vault are closed. |

### Escrow Account State

```rust
pub struct Escrow {
    pub seed: u64,      // Unique seed for PDA derivation
    pub maker: Pubkey,  // Maker's public key
    pub mint_a: Pubkey, // Token the maker deposited
    pub mint_b: Pubkey, // Token the maker wants in return
    pub receive: u64,   // Amount of token B the maker expects
    pub bump: u8,       // PDA bump seed
}
```

### PDA Seeds

- **Escrow**: `["escrow", maker_pubkey, seed_as_le_bytes]`
- **Vault**: Associated token account owned by the escrow PDA

## Prerequisites

- Rust 1.89.0+
- Solana CLI
- Anchor CLI

## Build

```sh
anchor build
```

## Test

Rust integration tests using LiteSVM:

```sh
cargo test --package anchor-escrow-q2-2026 --test test_initialize -- --nocapture
```

Tests cover:
- **Make + Refund** — maker deposits, then reclaims tokens
- **Make + Take** — maker deposits, taker completes the swap

### Test Output

```
running 2 tests

Make transaction successful
CUs consumed: 53496
Tx Signature: 3QbpNqzYkQEB9UE5ssQ7T6DVavXRXuQeMASZP6bQb6M6fkDZcDAiDP6hfdHE27CTzJUbcNLp3ktRR6ppgTW4CPys

 Refund Transaction Successful
CUs consumed: 29885
Tx signature: XpNM13PP7UsHBVrRCGyqbCd3vyhwRpXBmpMp38j9qPCDzQ8CERY81rRmcsqDEGCZrZF3mNCLso6HSCDk5YuNghG
test tests::test_make_and_refund ... ok

Make transaction successful
CUs Consumed: 50496
Tx signature: 2XRBQti1FBGgeusRZKwXHuKwM5X1Y6vvRgkvmPjnMHcC6Nn7gBFEhEkyc8qyvt3eyDgro79Y9fR4fLtghZpZ1xkk

Take Transaction successful
CUs consumed: 47814
Tx signature: 3L9qkFdAcYfyAEr9yTBsHcvBrMwuT9FWvkmCY83uadVzW3trF3j4XuRQqUhNjaqkRqqhFsUaUqkCuACG9n1exqU8
test tests::test_make_and_take ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## Project Structure

```
programs/anchor-escrow-q2-2026/
├── src/
│   ├── lib.rs            # Program entrypoint and instruction handlers
│   ├── state.rs          # Escrow account definition
│   ├── constants.rs      # PDA seed constants
│   ├── error.rs          # Custom error codes
│   └── instructions/
│       ├── make.rs       # Make instruction — deposit tokens
│       ├── take.rs       # Take instruction — fulfill the swap
│       └── refund.rs     # Refund instruction — reclaim tokens
└── tests/
    └── test_initialize.rs  # LiteSVM integration tests
```
