import wallet from "../../devnet-wallet.json";
import { 
    appendTransactionMessageInstruction, 
    appendTransactionMessageInstructions, 
    assertIsTransactionWithBlockhashLifetime, 
    createKeyPairSignerFromBytes, 
    createSolanaRpc, 
    createSolanaRpcSubscriptions, 
    createTransactionMessage, 
    generateKeyPairSigner, 
    getSignatureFromTransaction, 
    sendAndConfirmTransactionFactory, 
    setTransactionMessageFeePayerSigner, 
    setTransactionMessageLifetimeUsingBlockhash, 
    signTransactionMessageWithSigners
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import { getInitializeMint2Instruction, getInitializeMintInstruction, getMintSize, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

(async () => {
    try{
        const signer = await createKeyPairSignerFromBytes(
            new Uint8Array(wallet)
        );
        const mint = await generateKeyPairSigner();

        const space = BigInt(getMintSize());

        const rent = await rpc.getMinimumBalanceForRentExemption(space).send();
        const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
        
        const sendAndConfirm = sendAndConfirmTransactionFactory({
            rpc, rpcSubscriptions
        });

        const msg = createTransactionMessage({ version: 0 });

        const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

        const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
            latestBlockhash,
            msgWithPayer
        )

        const txMessage = appendTransactionMessageInstructions(
            [
                getCreateAccountInstruction({
                    payer: signer,
                    newAccount: mint,
                    lamports: rent,
                    space,
                    programAddress: TOKEN_PROGRAM_ADDRESS,
                }),

                getInitializeMintInstruction({
                    mint: mint.address,
                    decimals: 6,
                    mintAuthority: signer.address,
                }),
            ],
            msgWithLifetime
        )

        const signedTx = await signTransactionMessageWithSigners(txMessage);

        assertIsTransactionWithBlockhashLifetime(signedTx);

        const signature = getSignatureFromTransaction(signedTx);

        await sendAndConfirm(signedTx, { commitment: "confirmed" });

        console.log(`mint address: ${mint.address}. Transaction Signature: ${signature}`);

    } catch (error) {
        console.log(error);
    }
})();

// output from the program:
// mint address: J3gZVbwfvkEKD7C1CDe7829gyEjmCcPQL38hCdLZvtAF. 
// Transaction Signature: 3woEppAY87oJqZtUnvfA7M57gpF6ERSmVWTCGp2Fiu6EiVSoMVzk9RvN3P6kXksCHRqLQiZKPEuLNTq8dh3mVMJ