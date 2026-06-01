# NFT Marketplace — Solana Anchor Program

A fully on-chain NFT marketplace built with **Anchor 0.32.1** on Solana. Supports listing, buying (SOL and SPL tokens), counter-offers, delisting, and admin fee withdrawal — all using **Metaplex Core** (mpl-core) NFTs.

**Program ID:** `CiQM2wDGJBymguyD874w8uSPSrvFK8jnhnTAbQShKfTQ`

## Features

- **Initialize Marketplace** — Admin creates a named marketplace with a configurable fee (basis points) and a treasury PDA for collecting fees. A rewards mint is created for buyer incentives.
- **List NFT** — Sellers list mpl-core assets at a set price. Supports optional `payment_mint` for SPL token pricing. NFT ownership transfers to the listing PDA.
- **Buy with SOL** — Buyers purchase listed NFTs with SOL. Price splits between maker and treasury (fee). Buyer receives the NFT + a reward token.
- **Buy with SPL Token** — Same as above, but pays with any SPL token (e.g., USDC). The listing must specify a `payment_mint`.
- **Make Offer** — Buyers submit counter-offers below listing price. SOL is escrowed in the offer PDA.
- **Accept Offer** — Sellers accept an offer. Escrowed SOL transfers to maker (minus fee to treasury), NFT transfers to buyer, and buyer gets reward tokens.
- **Cancel Offer** — Buyers reclaim escrowed SOL from an unaccepted offer.
- **Delist** — Sellers withdraw their NFT from the marketplace, closing the listing.
- **Withdraw Fee** — Admin withdraws accumulated fees from the treasury PDA.

## Architecture

### Program Accounts (PDAs)

| Account | Seeds | Purpose |
|---------|-------|---------|
| **Marketplace** | `["marketplace", name]` | Stores admin, fee (bps), name, and PDA bumps |
| **Treasury** | `["treasury", marketplace]` | Collects marketplace fees (SOL) |
| **Rewards Mint** | `["rewards", marketplace]` | SPL mint for buyer reward tokens (6 decimals) |
| **Listing** | `["listing", asset]` | Stores maker, asset, price, payment_mint; holds NFT ownership |
| **Offer** | `["offer", asset, buyer]` | Stores buyer, asset, amount; holds escrowed SOL |

### Instruction Flow

```
                    ┌─────────────┐
                    │ Initialize  │
                    │ Marketplace │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  List NFT   │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
     ┌──────▼──────┐ ┌────▼─────┐ ┌──────▼──────┐
     │  Buy (SOL)  │ │Buy Token │ │ Make Offer  │
     └─────────────┘ └──────────┘ └──────┬──────┘
                                         │
                                  ┌──────┴──────┐
                                  │             │
                           ┌──────▼───┐  ┌──────▼──────┐
                           │  Accept  │  │   Cancel    │
                           │  Offer   │  │   Offer     │
                           └──────────┘  └─────────────┘

     Seller can Delist at any time ──► NFT returned to maker
     Admin can Withdraw Fees ──► SOL from treasury to admin
```

### Fee Model

- Fees are specified in **basis points** (e.g., `500` = 5%)
- On every sale (`buy`, `buy_with_token`, `accept_offer`): `fee = price × marketplace.fee / 10000`
- Fee goes to treasury (SOL) or treasury ATA (SPL tokens)
- Maker receives `price - fee`
- Checked arithmetic throughout to prevent overflow

## Tech Stack

| Component | Tool |
|-----------|------|
| Framework | Anchor 0.32.1 |
| NFT Standard | Metaplex Core (mpl-core 0.11.2) |
| Token Support | anchor-spl 0.32.1 (Token Interface) |
| Testing | LiteSVM 0.10.0 (local Solana VM) |
| Language | Rust (edition 2021) |

## Project Structure

```
programs/nft_marketplace/
├── src/
│   ├── lib.rs                    # Program entrypoint + instruction dispatch
│   ├── state.rs                  # Marketplace, Listing, Offer accounts
│   ├── error.rs                  # Custom error codes
│   └── instructions/
│       ├── initialize.rs         # Create marketplace + treasury + rewards mint
│       ├── list.rs               # List NFT with price (SOL or token)
│       ├── buy.rs                # Buy with SOL
│       ├── buy_with_token.rs     # Buy with SPL token
│       ├── delist.rs             # Withdraw NFT from marketplace
│       ├── withdraw_fee.rs       # Admin withdraws treasury fees
│       ├── make_offer.rs         # Buyer places counter-offer (SOL escrowed)
│       ├── accept_offer.rs       # Seller accepts offer
│       └── cancel_offer.rs       # Buyer cancels and reclaims SOL
├── tests/
│   ├── tests.rs                  # Integration tests (LiteSVM)
│   ├── fixtures/
│   │   └── mpl_core.so           # mpl-core program binary for local testing
│   └── ix_handlers/              # Test instruction builders
│       ├── initialize.rs
│       ├── list.rs
│       ├── buy.rs
│       ├── buy_with_token.rs
│       ├── delist.rs
│       ├── withdraw_fee.rs
│       ├── make_offer.rs
│       ├── accept_offer.rs
│       └── cancel_offer.rs
└── Cargo.toml
```

## Building

```bash
anchor build
```

## Testing

Tests use **LiteSVM** — a fast local Solana VM that runs entirely in-process. No validator needed.

```bash
cargo test
```

### Test Output

```
Running unittests src/lib.rs (target/debug/deps/nft_marketplace-26d8fe50c3196aab)

running 1 test
test test_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/tests.rs (target/debug/deps/tests-7e3cc5e1b50c0ff1)

running 7 tests
test test_list ... ok
test test_buy ... ok
test test_withdraw_fee ... ok
test test_initialize ... ok
test test_delist ... ok
test test_make_offer ... ok
test test_buy_with_token ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

   Doc-tests nft_marketplace

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Test Coverage

| Test | What it verifies |
|------|-----------------|
| `test_initialize` | Marketplace PDA creation with name, fee, treasury, and rewards mint |
| `test_list` | NFT listing with price, ownership transfer to listing PDA |
| `test_buy` | SOL purchase: payment split (maker + treasury fee), NFT transfer, reward mint |
| `test_buy_with_token` | SPL token purchase: creates payment mint, funds taker, verifies token transfers |
| `test_delist` | NFT returned to maker, listing account closed |
| `test_withdraw_fee` | Admin withdraws fees from treasury after a sale (rent-exempt check) |
| `test_make_offer` | Counter-offer creation with SOL escrowed in offer PDA |

## Error Codes

| Error | Description |
|-------|-------------|
| `NameTooLong` | Marketplace name exceeds 32 characters |
| `NameEmpty` | Marketplace name is empty |
| `InvalidFee` | Fee exceeds 10,000 basis points (100%) |
| `InvalidNftCollection` | NFT not from an allowed collection |
| `NotAdmin` | Caller is not the marketplace admin |
| `MarketplacePaused` | Marketplace is paused |
| `ArithmeticOverflow` | Overflow in fee calculation |
| `InsufficientTreasuryBalance` | Withdraw amount exceeds available treasury balance |
