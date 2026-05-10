import { address, appendTransactionMessageInstruction, appendTransactionMessageInstructions, assertIsTransactionWithBlockhashLifetime, createKeyPairSignerFromBytes, createSolanaRpc, createSolanaRpcSubscriptions, createTransactionMessage, getSignatureFromTransaction, sendAndConfirmTransactionFactory, setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash, signTransaction, signTransactionMessageWithSigners } from "@solana/kit";
import wallet from "../../devnet-wallet.json";
import { findAssociatedTokenPda, getCreateAssociatedTokenInstructionAsync, getMintToInstruction, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

const token_decimals = 1_000_000n;

const mint = address("J3gZVbwfvkEKD7C1CDe7829gyEjmCcPQL38hCdLZvtAF");

(async () => {

    try {
        const signer = await createKeyPairSignerFromBytes(
            new Uint8Array(wallet)
        );

        const [ata] = await findAssociatedTokenPda({
            mint,
            owner: signer.address,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
        })

        console.log(`Your ATA is : ${ata}`);

        const createAtaTx = await getCreateAssociatedTokenInstructionAsync({
            payer: signer,
            mint,
            owner: signer.address
        });

        const mintToIx = getMintToInstruction({
            mint,
            token: ata,
            mintAuthority: signer,
            amount: 1n * token_decimals,
        });

        const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

        const msg = createTransactionMessage({ version: 0 });
        
        const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

        const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
            latestBlockhash,
            msgWithPayer
        )

        const txMessage = appendTransactionMessageInstructions(
            [createAtaTx, mintToIx],
            msgWithLifetime
        )

        const signedTx = await signTransactionMessageWithSigners(txMessage);

        assertIsTransactionWithBlockhashLifetime(signedTx);
        
        const signature = getSignatureFromTransaction(signedTx);

        const sendAndConfirm = sendAndConfirmTransactionFactory({
                rpc, rpcSubscriptions
        });
        
        await sendAndConfirm(signedTx, { commitment: "confirmed" });
        console.log(`Mint txid: ${signature}`);
    } catch (error) {
        console.log(error);
    }
})();

// Output from the program:
// Your ATA is : 5FUfHoWRaUys6wFmBvRG9rrooBBCz4dJntkjKxH5uz3r
// Mint txid: 5QKAgamZ1Vtdh8aSjYazwrBtCZ4BfC1bhu4teYyV35vaEZdfirnTMcPXpRbeM3ZkvV5wMZkzBGEckpCigcXaYQ9R


