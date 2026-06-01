use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_list_ix(
    maker: &Address,
    asset: &Address,
    collection: Option<&Address>,
    listing: &Address,
    payment_mint: Option<&Address>,
    price: u64,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let mpl_core_program = pk(mpl_core::ID);
    let system_program = pk(anchor_lang::system_program::ID);

    let collection_key = collection.copied().unwrap_or(program_id);
    let payment_mint_key = payment_mint.copied().unwrap_or(program_id);

    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::List { price }.data(),
        vec![
            AccountMeta::new(*maker, true),
            AccountMeta::new(*asset, false),
            AccountMeta::new(collection_key, false),
            AccountMeta::new(*listing, false),
            AccountMeta::new_readonly(payment_mint_key, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    )
}
