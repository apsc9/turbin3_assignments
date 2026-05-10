# Turbin3 Assignment 2 — SPL Token & NFT Minting

Week 2 assignment: mint a custom SPL token and an MPL Core NFT on Solana devnet.

---

## Tasks Completed

- [x] Mint SPL token with on-chain metadata
- [x] Upload NFT image to Irys (decentralized storage)
- [x] Upload NFT metadata JSON to Irys
- [x] Mint NFT using MPL Core

---

## Tech Stack

| Package | Purpose |
|---|---|
| `@solana/kit` | Low-level Solana RPC, transaction building |
| `@solana-program/token` | SPL token instructions (ATA, mint, transfer) |
| `@solana-program/system` | Create account instruction |
| `@metaplex-foundation/mpl-token-metadata` | On-chain metadata for SPL token |
| `@metaplex-foundation/mpl-core` | MPL Core NFT standard |
| `@metaplex-foundation/umi` | Metaplex transaction abstraction layer |
| `@metaplex-foundation/umi-uploader-irys` | Decentralized file/metadata uploads |

---

## SPL Token

### Step 1 — Create Mint Account

[src/spl/spl_init.ts](src/spl/spl_init.ts)

Creates a new mint account on devnet with 6 decimals. Uses `@solana/kit` directly for transaction building.

```
Mint address: J3gZVbwfvkEKD7C1CDe7829gyEjmCcPQL38hCdLZvtAF
Tx: 3woEppAY87oJqZtUnvfA7M57gpF6ERSmVWTCGp2Fiu6EiVSoMVzk9RvN3P6kXksCHRqLQiZKPEuLNTq8dh3mVMJ
```

### Step 2 — Attach On-Chain Metadata

[src/spl/spl_metadata.ts](src/spl/spl_metadata.ts)

Attaches token metadata using Metaplex `mpl-token-metadata`.

| Field | Value |
|---|---|
| Name | Bull coin |
| Symbol | BULL |
| URI | `https://arweave.net/123456` |

```
Tx: 4oMMTAjwcizhhm5MUAhXAxtdwFUBcCgaeWE71WJvzjqjnR39FxZg5wbHSEwMJtzydLUWQAGHwMCuHqz6TKPcvCbK
```

### Step 3 — Mint Tokens

[src/spl/spl_mint.ts](src/spl/spl_mint.ts)

Creates an Associated Token Account (ATA) and mints 1 BULL token to it.

```
ATA: 5FUfHoWRaUys6wFmBvRG9rrooBBCz4dJntkjKxH5uz3r
Tx: 5QKAgamZ1Vtdh8aSjYazwrBtCZ4BfC1bhu4teYyV35vaEZdfirnTMcPXpRbeM3ZkvV5wMZkzBGEckpCigcXaYQ9R
```

### Step 4 — Transfer Tokens

[src/spl/spl_transfer.ts](src/spl/spl_transfer.ts)

Creates recipient ATA and transfers 1 BULL token using `getTransferCheckedInstruction`.

```
From ATA: 5FUfHoWRaUys6wFmBvRG9rrooBBCz4dJntkjKxH5uz3r
To ATA:   4fdqoqWaHGMCCHngzy5LTXJLgvXf1kmZTqYeTdC5gKeY
Tx: 2nZqwtJoQ9me39Uj3EPEtt4U2AQpHEZiraesEqMy7f1Z9e2CM3Qk5AMiQC8Fuo5uH6cnXYQuNNznsuAx7YjZhTKK
```

---

## NFT (MPL Core)

### Step 1 — Upload Image

[src/nft/nft_image.ts](src/nft/nft_image.ts)

Reads `matrix.avif` from disk and uploads to Irys devnet via Metaplex UMI.

```
Image URI: https://gateway.irys.xyz/hSkaGC1zLyDShaU3NzzqnLVC2iddPn4sYscAcf5AmEK
```

### Step 2 — Upload Metadata

[src/nft/nft_metadata.ts](src/nft/nft_metadata.ts)

Constructs and uploads JSON metadata (Metaplex standard) to Irys.

```json
{
  "name": "Neo",
  "description": "the matrix is you",
  "image": "https://gateway.irys.xyz/hSkaGC1zLyDShaU3NzzqnLVC2iddPn4sYscAcf5AmEK",
  "attributes": [{ "trait_type": "Rarity", "value": "Legendary" }]
}
```

```
Metadata URI: https://gateway.irys.xyz/BgpvwFW22KeAtBMC46fkML8jNKoxdxcK6YLHnnNeVXiE
```

### Step 3 — Mint NFT

[src/nft/nft_mint.ts](src/nft/nft_mint.ts)

Mints the NFT as an MPL Core asset using the `create` instruction from `@metaplex-foundation/mpl-core`.

```
Asset address: 4tBJ79t78tK4NVdpwdX6oe2RH6w4LU4MUssSBNd6trhS
Tx: QseKoBhUnCrd7mAudPasQux4irY4d8rFYqZMC27UZbGJSos8vw559nX9qZ8iRBa9NUy4HryWJM3JMMRTjYJqn2A
```

---

## Running Locally

```bash
# Install dependencies
npm install

# SPL token flow
npm run spl:init
npm run spl:metadata
npm run spl:mint
npm run spl:transfer

# NFT flow
npm run nft:image
npm run nft:metadata
npm run nft:mint
```

> **Prerequisites:** `devnet-wallet.json` must be present in the project root with a funded devnet wallet.  
> Get devnet SOL: `solana airdrop 2 <your-address> --url devnet`
