import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { DiceGame } from "../target/types/dice_game";
import { expect } from "chai";
import {
    Transaction,
    Keypair,
    PublicKey,
    SystemProgram,
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction,
} from "@solana/web3.js";
import { createHash } from "crypto";
import { BN } from "bn.js";

const commitment = "confirmed";

const confirmTx = async (
  connection: anchor.web3.Connection,
  signature: string,
  operationLabel: string,
) => {
  const latestBlockHash = await connection.getLatestBlockhash();

  await connection.confirmTransaction(
    {
      signature,
      ...latestBlockHash,
    },
    commitment,
  );
  console.log(`${operationLabel} signature: ${signature}`);
};

describe("dice-game", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DiceGame as Program<DiceGame>;
  const house = Keypair.generate();
  const player = Keypair.generate();
  const seed = new BN(Date.now());

  const vaultPda = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), house.publicKey.toBuffer()],
    program.programId
  )[0];

  let betPda: PublicKey;

  it("airdrop", async () => {
    await Promise.all(
      [house, player].map(async (key) => {
        return await provider.connection
          .requestAirdrop(key.publicKey, 100 * LAMPORTS_PER_SOL)
          .then((sig) => confirmTx(provider.connection, sig, "airdrop"));
      }),
    );
  });

  it("initializes the vault", async () => {
    const amount = new BN(10 * LAMPORTS_PER_SOL);

    const tx = await program.methods
      .initialize(amount)
      .accounts({
        house: house.publicKey,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([house])
      .rpc();

    console.log("Initialize tx:", tx);

    const vaultBalance = await provider.connection.getBalance(vaultPda);
    expect(vaultBalance).to.equal(amount.toNumber());
  });

  it("places a bet", async () => {
    const roll = 50;
    const amount = new BN(LAMPORTS_PER_SOL / 10); // 0.1 SOL

    [betPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("bet"),
        vaultPda.toBuffer(),
        player.publicKey.toBuffer(),
        seed.toArrayLike(Buffer, "le", 16),
      ],
      program.programId
    );

    const tx = await program.methods
      .placeBet(seed, roll, amount)
      .accounts({
        player: player.publicKey,
        house: house.publicKey,
        vault: vaultPda,
        bet: betPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    console.log("Place bet tx:", tx);

    const betAccount = await program.account.bet.fetch(betPda);
    expect(betAccount.player.toBase58()).to.equal(player.publicKey.toBase58());
    expect(betAccount.roll).to.equal(roll);
    expect(betAccount.amount.toNumber()).to.equal(amount.toNumber());
  });

  it("resolves a bet using instruction introspection with custom struct", async () => {
    const betAccount = await program.account.bet.fetch(betPda);

    const resultRoll = 30;

    // Build the bet serialized data matching Bet::to_slice()
    const betData = Buffer.concat([
      betAccount.player.toBuffer(),                                      
      new BN(betAccount.seed).toArrayLike(Buffer, "le", 16),            
      new BN(betAccount.slot.toString()).toArrayLike(Buffer, "le", 8),   
      new BN(betAccount.amount.toString()).toArrayLike(Buffer, "le", 8), 
      Buffer.from([betAccount.roll, betAccount.bump]),                   
    ]);

    // Compute SHA-256 hash of (bet_data ++ result_roll)
    const hashInput = Buffer.concat([betData, Buffer.from([resultRoll])]);
    const resultHash = createHash("sha256").update(hashInput).digest();

    const clock = await provider.connection.getAccountInfo(
      new PublicKey("SysvarC1ock11111111111111111111111111111111")
    );
    const timestamp = clock!.data.readBigInt64LE(32);

    // Build the submit_resolution instruction manually
    const submitResolutionIx = await program.methods
      .submitResolution({
        betKey: betPda,
        resultRoll: resultRoll,
        timestamp: new BN(timestamp.toString()),
        resultHash: Array.from(resultHash),
      })
      .accounts({
        house: house.publicKey,
      })
      .instruction();

    // Build the resolve_bet instruction
    const resolveBetIx = await program.methods
      .resolveBet()
      .accounts({
        house: house.publicKey,
        player: player.publicKey,
        bet: betPda,
      })
      .instruction();

    // Send both instructions in one transaction — introspection reads ix[0]
    const tx = new Transaction().add(submitResolutionIx).add(resolveBetIx);

    const sig = await sendAndConfirmTransaction(
      provider.connection,
      tx,
      [house],
      { commitment }
    );

    console.log("Resolve bet tx:", sig);

    const closedBet = await provider.connection.getAccountInfo(betPda);
    expect(closedBet).to.be.null;

    console.log("Bet resolved successfully via instruction introspection with custom HouseResolution struct!");
  });
});
