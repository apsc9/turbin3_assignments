# Dice Game — Solana Anchor Program with Instruction Introspection

A provably fair dice game built on Solana using Anchor 1.0. The program demonstrates **instruction introspection** with a **custom `HouseResolution` struct** — instead of relying on Ed25519 signature verification, the house commits dice results via a custom serialized struct that is verified on-chain by inspecting a preceding instruction in the same transaction.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Transaction                            │
│                                                             │
│  ┌─────────────────────┐    ┌─────────────────────────┐    │
│  │  submit_resolution  │───▶│      resolve_bet        │    │
│  │  (HouseResolution)  │    │  (introspects ix[0])    │    │
│  └─────────────────────┘    └─────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

The `resolve_bet` instruction reads the preceding `submit_resolution` instruction from the same transaction via the Instructions Sysvar, deserializes the custom `HouseResolution` struct, and cryptographically verifies the result before settling the bet.

## Program Instructions

| Instruction | Signer | Description |
|-------------|--------|-------------|
| `initialize` | House | Funds the vault with initial liquidity |
| `place_bet` | Player | Creates a bet account and deposits the wager |
| `resolve_bet` | House | Verifies the resolution and settles the bet |
| `submit_resolution` | House | Commits the `HouseResolution` struct (no-op, exists for introspection) |
| `refund_bet` | Player | Refunds the bet if the house fails to resolve within timeout |

## Program Derived Addresses (PDAs)

### Vault PDA

```
Seeds: ["vault", house_pubkey]
```

Holds all deposited SOL (house liquidity + player wagers). Controlled by the program — payouts are signed with PDA signer seeds.

### Bet PDA

```
Seeds: ["bet", vault_pubkey, player_pubkey, seed_u128_le_bytes]
```

Stores the bet state: player, seed, slot, amount, roll target, and bump. The `seed` (u128) ensures a player can have multiple concurrent bets. The account is closed (rent returned to player) when resolved or refunded.

## Custom Struct: `HouseResolution`

```rust
pub struct HouseResolution {
    pub bet_key: Pubkey,       // 32 bytes — which bet this resolves
    pub result_roll: u8,       //  1 byte  — the dice result (1-100)
    pub timestamp: i64,        //  8 bytes — when the resolution was created
    pub result_hash: [u8; 32], // 32 bytes — SHA-256 commitment proof
}
```

### Hash Commitment

The `result_hash` field is computed as:

```
SHA-256(bet_serialized_data || result_roll)
```

Where `bet_serialized_data` is the deterministic serialization of the on-chain Bet account (player + seed + slot + amount + roll + bump). This binds the result to the immutable bet state, preventing the house from replaying or tampering with resolutions.

## Instruction Introspection Flow

1. **House builds a transaction** with two instructions:
   - `submit_resolution(HouseResolution { ... })` — at index 0
   - `resolve_bet()` — at index 1

2. **`resolve_bet` executes** and performs introspection:
   ```
   load_instruction_at_checked(0, instructions_sysvar)
   ```

3. **Verification chain:**
   - Program ID of ix[0] must equal our program ID
   - Deserialize `HouseResolution` from ix[0] data (skip 8-byte Anchor discriminator)
   - `resolution.bet_key` must match the bet account being resolved
   - `resolution.result_roll` must be in range [1, 100]
   - `resolution.result_hash` must equal `SHA-256(bet.to_slice() || result_roll)`

4. **Settlement:**
   - If `result_roll <= bet.roll` → player wins, vault pays out
   - Payout formula: `(amount * (10000 - 150) * 100) / (roll * 10000)` (1.5% house edge)
   - Bet account is closed regardless of outcome (rent returned to player)

## Why Instruction Introspection?

Instead of using Ed25519 signature verification (the typical pattern), this program demonstrates that **any custom data structure** can be verified through instruction introspection. The key insight:

- The Instructions Sysvar gives on-chain programs access to **all instructions** in the current transaction
- By requiring a specific preceding instruction with known structure, the program can enforce that certain data was committed atomically
- The SHA-256 hash commitment ensures the house cannot manipulate the result after seeing the bet state

This pattern is generalizable to any scenario where you need to verify structured data was committed in the same atomic transaction.

## Game Mechanics

- **Roll range:** 1–99 (player chooses a target)
- **Win condition:** `result_roll <= player_roll` (rolling under)
- **Odds:** Higher roll target = higher chance of winning, lower payout
- **House edge:** 1.5% (150 basis points)
- **Minimum bet:** 0.01 SOL (10,000,000 lamports)
- **Timeout refund:** If house doesn't resolve within 1000 slots (~7 min), player can self-refund

## State Account: `Bet`

```rust
pub struct Bet {
    pub player: Pubkey,  // 32 bytes
    pub seed: u128,      // 16 bytes — unique identifier
    pub slot: u64,       //  8 bytes — slot when bet was placed
    pub amount: u64,     //  8 bytes — wager in lamports
    pub roll: u8,        //  1 byte  — target roll (1-99)
    pub bump: u8,        //  1 byte  — PDA bump
}
```

## Error Codes

| Error | Description |
|-------|-------------|
| `MinimumBet` | Bet amount below 0.01 SOL |
| `MinimumRoll` | Roll target below 1 |
| `MaximumRoll` | Roll target above 99 |
| `Overflow` | Arithmetic overflow in payout calculation |
| `TimeoutNotReached` | Refund attempted before 1000 slots elapsed |
| `MissingResolutionInstruction` | No preceding instruction found |
| `InvalidResolutionProgram` | Preceding instruction not from this program |
| `InvalidResolutionData` | Could not deserialize HouseResolution |
| `ResolutionBetMismatch` | Resolution targets a different bet |
| `InvalidResultRoll` | Result roll outside 1-100 range |
| `InvalidResolutionHash` | Hash commitment verification failed |

## Building & Testing

```bash
anchor build
anchor test
```

## Test Results

```
  dice-game
airdrop signature: 2CRfcwaygWB7ujrjSsMQVzNviPsyrByh6JZNDAtDF8uXaB1dkXMLDx1Ts1N4DEFyLavnW4e4yKgQJB3Hw5G3iL7N
airdrop signature: 5roLvSbfrA5mshgTdggqWcz94GENm53PAxtwVPBPGNH4L47Z4uVQPEjwDJULfRGck3czC3rzhXedBxjPEwZHvFk6
    ✔ airdrop (44ms)
Initialize tx: 5AakL897bn7HzDNRQ3wvr4qCLHkSM4b28Q8forMYK4yxRMDN1QCSwh5QGTd1xpUzh7rmxmi6z6pCyJYLaVjfuUum
    ✔ initializes the vault
Place bet tx: 2o9gAuQfSxYUPkJA4DFgfCpy17Xz2nvz6YKsW6n97o2ePK64fVvyus1FUjNQnzUcwgdC22UVHbwpRcnQnpkHpfUN
    ✔ places a bet
Resolve bet tx: 2VXC6btP6cNomcTrFFZUc2GiDoF3bNQqRHugdtLpp4FXt4ey9PxgPqw3pxwBLTMrmURYUjHWwMSSwA5UJLDQ6gaY
Bet resolved successfully via instruction introspection with custom HouseResolution struct!
    ✔ resolves a bet using instruction introspection with custom struct


  4 passing (103ms)
```
