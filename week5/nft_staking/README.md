# NFT Staking Program

A Solana program built with Anchor that lets users stake Metaplex Core NFTs to earn SPL token rewards. The program manages the full lifecycle — collection creation, minting, staking with freeze-locking, time-based reward accumulation, partial claiming, and unstaking with automatic final reward payout.

## Program ID

```
HrqAjT6Q9X7EHxKhHdXtxi7ZKnHQk6DNcoAVQFLmSse3
```

## How It Works

### Staking Flow

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Initialize  │────▶│   Create     │────▶│  Mint Asset   │────▶│    Stake     │
│  (admin)     │     │  Collection  │     │  (user)       │     │  (user)      │
└─────────────┘     └─────────────┘     └──────────────┘     └──────┬──────┘
                                                                     │
                                                          NFT frozen │ rewards accrue
                                                                     │
                                                          ┌──────────▼──────────┐
                                                          │                     │
                                                    ┌─────┴─────┐       ┌──────┴──────┐
                                                    │   Claim    │       │   Unstake    │
                                                    │  Rewards   │       │ (after       │
                                                    │ (anytime)  │       │  freeze)     │
                                                    └────────────┘       └─────────────┘
```

1. **Admin initializes** the staking config — sets rewards rate (basis points) and freeze period (days). This also creates the rewards SPL token mint.
2. **Collection is created** as a Metaplex Core collection with a PDA as the update authority.
3. **Users mint NFTs** into the collection.
4. **Staking** freezes the NFT (via the FreezeDelegate plugin) so it can't be transferred, and writes `staked=true`, `staked_at`, and `last_claimed` timestamps into the asset's on-chain Attributes plugin.
5. **Claiming rewards** can happen anytime while staked. Rewards are calculated from `last_claimed` to now, and `last_claimed` is reset — preventing double-claiming.
6. **Unstaking** requires the freeze period to have elapsed (checked against `staked_at`). It pays out any remaining rewards since `last_claimed`, unfreezes the NFT, resets staking attributes, and decrements the collection's `staked_count`.

### Rewards Formula

```
rewards = floor(days_elapsed) × rewards_bps × 10^decimals / 10000
```

Where:
- **days_elapsed** = `floor((current_timestamp - last_claimed) / 86400)` — whole days only, no partial-day rewards
- **rewards_bps** = configurable rate in basis points (e.g., 100 bps = 1%)
- **decimals** = rewards mint decimals (set to 6)

**Example:** With `rewards_bps = 500` (5%) and 10 days staked:
```
rewards = 10 × 500 × 1_000_000 / 10_000 = 500_000_000 (500 tokens with 6 decimals)
```

### Collection-Level Tracking

The program tracks a `staked_count` attribute on the collection itself — incremented on stake, decremented on unstake. This enables on-chain queries for how many NFTs are currently staked in a collection.

## PDA Accounts

| Account | Seeds | Description |
|---------|-------|-------------|
| **Config** | `["config", collection_pubkey]` | Stores `rewards_bps`, `freeze_period`, `rewards_bump`, `bump` |
| **Update Authority** | `["update_authority", collection_pubkey]` | PDA that acts as the collection's update authority — signs for attribute and plugin modifications |
| **Rewards Mint** | `["rewards_mint", config_pubkey]` | SPL token mint for reward tokens. Mint authority is the Config PDA |

### Signing Authority

- **Config PDA** — signs `mint_to` CPI calls to mint reward tokens
- **Update Authority PDA** — signs Metaplex Core CPIs to add/update Attributes and FreezeDelegate plugins

## On-Chain State

### Config Account

| Field | Type | Description |
|-------|------|-------------|
| `rewards_bps` | `u16` | Reward rate in basis points per day |
| `freeze_period` | `u16` | Minimum staking duration in days before unstaking is allowed |
| `rewards_bump` | `u8` | Bump seed for the rewards mint PDA |
| `bump` | `u8` | Bump seed for the config PDA |

### Asset Attributes (stored via Metaplex Core Attributes Plugin)

| Key | Values | Description |
|-----|--------|-------------|
| `staked` | `"true"` / `"false"` | Whether the NFT is currently staked |
| `staked_at` | Unix timestamp string | When the NFT was staked (used for freeze period check) |
| `last_claimed` | Unix timestamp string | Last time rewards were claimed (used for reward calculation) |

### Collection Attributes

| Key | Values | Description |
|-----|--------|-------------|
| `staked_count` | Integer string | Number of NFTs currently staked in this collection |

## Architecture

```
programs/nft_staking/src/
├── lib.rs                  # Program entrypoint and instruction dispatch
├── constants.rs            # Seed constants
├── error.rs                # Custom error codes
├── state/
│   └── config.rs           # Config account struct
└── instructions/
    ├── initialize.rs       # Initialize config + rewards mint
    ├── create_collection.rs # Create Metaplex Core collection with PDA authority
    ├── mint_asset.rs       # Mint NFT into collection
    ├── stake.rs            # Freeze NFT, write staking attributes, increment staked_count
    ├── unstake.rs          # Thaw NFT, pay remaining rewards, decrement staked_count
    └── claim_rewards.rs    # Mint reward tokens, update last_claimed
```

## Error Codes

| Error | Description |
|-------|-------------|
| `InvalidOwner` | Signer is not the asset owner |
| `InvalidUpdateAuthority` | Update authority mismatch |
| `AlreadyStaked` | Attempting to stake an already-staked NFT |
| `AssetNotStaked` | Attempting to unstake/claim on a non-staked NFT |
| `InvalidTimestamp` | Timestamp parsing or arithmetic failure |
| `FreezePeriodNotElapsed` | Unstake attempted before freeze period ends |
| `InvalidRewardsBps` | Reward calculation overflow |
| `NoRewardsToClaim` | Claim called with zero elapsed days |
| `Overflow` | Arithmetic overflow (e.g., staked_count) |

## Tech Stack

- **Anchor** 0.31.1
- **Metaplex Core** (mpl-core 0.11.1) for NFT operations (collections, assets, plugins)
- **LiteSVM** for fast Rust-native testing (no validator needed)

## Building

```sh
anchor build
```

## Testing

Tests are written in Rust using LiteSVM:

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

All 11 tests passing — covers initialization, collection creation, minting, staking, unstaking (including freeze period enforcement), reward claiming, and edge cases (double-staking, zero-reward claim, claim-then-unstake, and stake-unstake-restake cycle).
