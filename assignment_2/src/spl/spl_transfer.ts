import { address, appendTransactionMessageInstructions, assertIsTransactionWithBlockhashLifetime, createKeyPairSignerFromBytes, createSolanaRpc, createSolanaRpcSubscriptions, createTransactionMessage, getSignatureFromTransaction, sendAndConfirmTransactionFactory, setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash, signTransactionMessageWithSigners } from "@solana/kit";
import wallet from "../../devnet-wallet.json";
import { findAssociatedTokenPda, getCreateAssociatedTokenInstructionAsync, getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";


const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

const mint = address("J3gZVbwfvkEKD7C1CDe7829gyEjmCcPQL38hCdLZvtAF");

const to = address("6QLH6XaUB5UYw96MAxhG5nvadjYes5aBR8RCX8VTzGmP");

(async () => {
    try {
        const signer = await createKeyPairSignerFromBytes(
            new Uint8Array(wallet)
        );

        const sendAndConfirm = sendAndConfirmTransactionFactory({
                rpc, rpcSubscriptions
        });

        const [fromAta] = await findAssociatedTokenPda({
            mint,
            owner: signer.address,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
        })
        console.log(`Your fromAta is : ${fromAta}`);

        const [toAta] = await findAssociatedTokenPda({
            mint,
            owner: to,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
        })
        console.log(`Your toAta is : ${toAta}`);

        const createAtaTx = await getCreateAssociatedTokenInstructionAsync({
            payer: signer,
            mint,
            owner: to
        });

        const transferTx = getTransferCheckedInstruction({
            source: fromAta,
            mint,
            destination: toAta,
            authority: signer,
            amount: 1_000_000n,
            decimals: 6,
        });

        const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
        
        const msg = createTransactionMessage({ version: 0 });
                
        const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

        const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
            latestBlockhash,
            msgWithPayer
        )

        const txMessage = appendTransactionMessageInstructions(
            [createAtaTx, transferTx],
            msgWithLifetime
        )

        const signedTx = await signTransactionMessageWithSigners(txMessage);

        assertIsTransactionWithBlockhashLifetime(signedTx);
        
        const signature = getSignatureFromTransaction(signedTx);
        
        await sendAndConfirm(signedTx, { commitment: "confirmed" });
        console.log(`Mint txid: ${signature}`);



    } catch (error) {
        console.log(error);
    }
})();

// output from the program:
// Your fromAta is : 5FUfHoWRaUys6wFmBvRG9rrooBBCz4dJntkjKxH5uz3r
// Your toAta is : 4fdqoqWaHGMCCHngzy5LTXJLgvXf1kmZTqYeTdC5gKeY
// Mint txid: 2nZqwtJoQ9me39Uj3EPEtt4U2AQpHEZiraesEqMy7f1Z9e2CM3Qk5AMiQC8Fuo5uH6cnXYQuNNznsuAx7YjZhTKK