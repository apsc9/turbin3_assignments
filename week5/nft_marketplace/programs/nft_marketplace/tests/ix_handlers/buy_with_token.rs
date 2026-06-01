use {
    anchor_lang::InstructionData,
    anchor_spl::associated_token,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_buy_with_token_ix(
    taker: &Address,
    maker: &Address,
    asset: &Address,
    collection: Option<&Address>,
    marketplace: &Address,
    listing: &Address,
    payment_mint: &Address,
    taker_ata: &Address,
    rewards_mint: &Address,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let mpl_core_program = pk(mpl_core::ID);
    let system_program = pk(anchor_lang::system_program::ID);
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());
    let associated_token_program = Address::from(associated_token::ID.to_bytes());

    let to_old_pk = |addr: &Address| {
        anchor_lang::prelude::Pubkey::new_from_array(addr.to_bytes())
    };

    let maker_ata = Address::from(
        associated_token::get_associated_token_address(
            &to_old_pk(maker),
            &to_old_pk(payment_mint),
        ).to_bytes(),
    );

    let treasury_ata = Address::from(
        associated_token::get_associated_token_address(
            &to_old_pk(marketplace),
            &to_old_pk(payment_mint),
        ).to_bytes(),
    );

    let taker_rewards_ata = Address::from(
        associated_token::get_associated_token_address(
            &to_old_pk(taker),
            &to_old_pk(rewards_mint),
        ).to_bytes(),
    );

    let collection_key = collection.copied().unwrap_or(program_id);

    // Account order matches BuyWithToken struct:
    // taker, maker, asset, collection, marketplace, listing,
    // payment_mint, taker_ata, maker_ata, treasury_ata,
    // rewards_mint, taker_rewards_ata, mpl_core_program,
    // associated_token_program, system_program, token_program
    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::BuyWithToken {}.data(),
        vec![
            AccountMeta::new(*taker, true),
            AccountMeta::new(*maker, false),
            AccountMeta::new(*asset, false),
            AccountMeta::new(collection_key, false),
            AccountMeta::new_readonly(*marketplace, false),
            AccountMeta::new(*listing, false),
            AccountMeta::new_readonly(*payment_mint, false),
            AccountMeta::new(*taker_ata, false),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new(treasury_ata, false),
            AccountMeta::new(*rewards_mint, false),
            AccountMeta::new(taker_rewards_ata, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(associated_token_program, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}
