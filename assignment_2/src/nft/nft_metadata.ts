import { createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi";
import wallet from "../../devnet-wallet.json";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";


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
        const image = "https://gateway.irys.xyz/hSkaGC1zLyDShaU3NzzqnLVC2iddPn4sYscAcf5AmEK";
        const metadata = {
            name: "Neo",
            description: "the matrix is you",
            image,
            attributes: [{ trait_type: "Rarity", value: "Legendary"}],

            properties: {
                files: [
                    {
                        type: "image/jpeg",
                        uri: image,
                    },
                ],
                category: "image",
            },
        };

        const myUri = await umi.uploader.uploadJson(metadata);
        console.log(`metadata uri: ${myUri}`);

    } catch (error) {
        console.log(error);
    }
})()

// output :
// metadata uri: https://gateway.irys.xyz/BgpvwFW22KeAtBMC46fkML8jNKoxdxcK6YLHnnNeVXiE