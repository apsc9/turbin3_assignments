# NFT Staking Program

A Solana program built with Anchor that enables NFT staking with reward distribution. Users can stake Metaplex Core NFTs, earn rewards over time based on a configurable basis-point rate, and unstake after a freeze period.

## Program ID

```
HrqAjT6Q9X7EHxKhHdXtxi7ZKnHQk6DNcoAVQFLmSse3
```

## Features

- **Initialize** — Set up the staking config with a rewards rate (basis points) and a freeze period (days)
- **Create Collection** — Create a Metaplex Core collection for stakeable NFTs
- **Mint Asset** — Mint a new NFT into the collection
- **Stake** — Stake an NFT to begin earning rewards
- **Unstake** — Unstake an NFT after the freeze period has elapsed
- **Claim Rewards** — Claim accumulated staking rewards (minted as SPL tokens)

## Architecture

```
programs/nft_staking/src/
├── lib.rs              # Program entrypoint and instruction dispatch
├── constants.rs        # Seed constants
├── error.rs            # Custom error codes
├── state/
│   └── config.rs       # Config account (rewards_bps, freeze_period, bumps)
└── instructions/
    ├── initialize.rs   # Initialize config + rewards mint
    ├── create_collection.rs
    ├── mint_asset.rs
    ├── stake.rs        # Stake NFT, record timestamp
    ├── unstake.rs      # Unstake after freeze period
    └── claim_rewards.rs # Mint reward tokens based on elapsed time
```

## Tech Stack

- **Anchor** 0.31.1
- **Metaplex Core** (mpl-core) for NFT operations
- **LiteSVM** for testing

## Building

```sh
anchor build
```

## Testing

Tests are written in Rust using LiteSVM (no validator required):

```sh
cargo test --manifest-path programs/nft_staking/Cargo.toml
```

### Test Results

```
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.60s
     Running unittests src/lib.rs (target/debug/deps/nft_staking-362c0500467fd5fb)

running 1 test
test test_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/tests.rs (target/debug/deps/tests-e2752a641e0a1d7b)

running 11 tests
test test_create_collection ... ok
test test_mint_asset ... ok
test test_initialize ... ok
test test_stake_already_staked_fails ... ok
test test_claim_rewards_no_time_elapsed_fails ... ok
test test_stake ... ok
test test_claim_rewards ... ok
test test_claim_then_unstake ... ok
test test_unstake_before_freeze_period_fails ... ok
test test_unstake ... ok
test test_stake_unstake_restake ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

   Doc-tests nft_staking

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

All 12 tests passing — covers initialization, minting, staking, unstaking (including freeze period enforcement), reward claiming, and edge cases like double-staking and restaking.
