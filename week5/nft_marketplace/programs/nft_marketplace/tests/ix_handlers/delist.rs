use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_delist_ix(
    maker: &Address,
    asset: &Address,
    collection: Option<&Address>,
    listing: &Address,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let mpl_core_program = pk(mpl_core::ID);
    let system_program = pk(anchor_lang::system_program::ID);

    let collection_key = collection.copied().unwrap_or(program_id);

    // Account order matches Delist struct:
    // maker, asset, collection, listing, mpl_core_program, system_program
    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::Delist {}.data(),
        vec![
            AccountMeta::new(*maker, true),
            AccountMeta::new(*asset, false),
            AccountMeta::new(collection_key, false),
            AccountMeta::new(*listing, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    )
}
