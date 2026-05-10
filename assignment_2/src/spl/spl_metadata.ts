import wallet from "../../devnet-wallet.json";
import { createSignerFromKeypair, publicKey, signerIdentity } from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { createMetadataAccountV3, CreateMetadataAccountV3InstructionAccounts, CreateMetadataAccountV3InstructionArgs, CreateMetadataAccountV3InstructionData, DataV2Args } from "@metaplex-foundation/mpl-token-metadata";
import bs58 from "bs58";

const mint = publicKey("J3gZVbwfvkEKD7C1CDe7829gyEjmCcPQL38hCdLZvtAF");

const umi = createUmi("https://api.devnet.solana.com");

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));

(async () => {
    try {

        const accounts: CreateMetadataAccountV3InstructionAccounts = {
            mint,
            mintAuthority: signer,
        }

        const data: DataV2Args = {
            name: "Bull coin",
            symbol: "Bull",
            uri: "https://arweave.net/123456",
            sellerFeeBasisPoints: 1,
            creators: null,
            collection: null,
            uses: null,
        }

        const args: CreateMetadataAccountV3InstructionArgs = {
            data,
            isMutable: true,
            collectionDetails: null,
        }
        const tx = createMetadataAccountV3(umi , {
            ...accounts,
            ...args,
        })

        const result = await tx.sendAndConfirm(umi);
        console.log(bs58.encode(Buffer.from(result.signature)));
        

    } catch (error){
        console.log("error:", error);
    }
})();

// signature got from  above program:
// 4oMMTAjwcizhhm5MUAhXAxtdwFUBcCgaeWE71WJvzjqjnR39FxZg5wbHSEwMJtzydLUWQAGHwMCuHqz6TKPcvCbK