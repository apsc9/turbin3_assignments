use {
    anchor_lang::InstructionData,
    anchor_spl::associated_token,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_buy_ix(
    taker: &Address,
    maker: &Address,
    asset: &Address,
    collection: Option<&Address>,
    marketplace: &Address,
    listing: &Address,
    treasury: &Address,
    rewards_mint: &Address,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let mpl_core_program = pk(mpl_core::ID);
    let system_program = pk(anchor_lang::system_program::ID);
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());
    let associated_token_program = Address::from(associated_token::ID.to_bytes());

    let taker_rewards_ata = Address::from(
        associated_token::get_associated_token_address(
            &anchor_lang::prelude::Pubkey::new_from_array(taker.to_bytes()),
            &anchor_lang::prelude::Pubkey::new_from_array(rewards_mint.to_bytes()),
        )
        .to_bytes(),
    );

    let collection_key = collection.copied().unwrap_or(program_id);

    // Account order matches Buy struct:
    // taker, maker, asset, collection, marketplace, listing, treasury,
    // rewards_mint, taker_rewards_ata, mpl_core_program,
    // associated_token_program, system_program, token_program
    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::Buy {}.data(),
        vec![
            AccountMeta::new(*taker, true),
            AccountMeta::new(*maker, false),
            AccountMeta::new(*asset, false),
            AccountMeta::new(collection_key, false),
            AccountMeta::new_readonly(*marketplace, false),
            AccountMeta::new(*listing, false),
            AccountMeta::new(*treasury, false),
            AccountMeta::new(*rewards_mint, false),
            AccountMeta::new(taker_rewards_ata, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(associated_token_program, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}
