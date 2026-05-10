import { createSignerFromKeypair, generateSigner, signerIdentity } from "@metaplex-foundation/umi";
import wallet from "../../devnet-wallet.json";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { create, mplCore } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";


const umi = createUmi("https://api.devnet.solana.com");

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));

umi.use(mplCore());

(async () => {
    try {
        const metadataUri = "https://gateway.irys.xyz/BgpvwFW22KeAtBMC46fkML8jNKoxdxcK6YLHnnNeVXiE";

        const asset = generateSigner(umi);

        const tx = await create(umi, {
            asset,
            name: "Neo",
            uri: metadataUri,
        }).sendAndConfirm(umi);

        const signature = base58.deserialize(tx.signature)[0];
        console.log(`signature ${signature}, asset: ${asset.publicKey}`);
        
    } catch (e) {
        console.log(`error is : ${e}`);
    }
})();

// output :
// signature QseKoBhUnCrd7mAudPasQux4irY4d8rFYqZMC27UZbGJSos8vw559nX9qZ8iRBa9NUy4HryWJM3JMMRTjYJqn2A
// asset: 4tBJ79t78tK4NVdpwdX6oe2RH6w4LU4MUssSBNd6trhS
// nft @ https://core.metaplex.com/explorer/4tBJ79t78tK4NVdpwdX6oe2RH6w4LU4MUssSBNd6trhS?env=devnet