import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json";
import { createGenericFile, createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys"
import { readFile } from "fs/promises";


const umi = createUmi("https://api.devnet.solana.com");

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(
    irysUploader({
        address:"https://devnet.irys.xyz",
    })
);

umi.use(signerIdentity(signer));

(async () => {
    try {
        const image = await readFile("./matrix.avif");

        const file = createGenericFile(image, "matrix.avif", {
            contentType: "image/jpeg",
        });

        const [myUri] = await umi.uploader.upload([file]);
        console.log("Your image URI: ", myUri);

    } catch (error) {
        console.log(error);
    }
})()

// output :
// Your image URI:  https://gateway.irys.xyz/hSkaGC1zLyDShaU3NzzqnLVC2iddPn4sYscAcf5AmEK