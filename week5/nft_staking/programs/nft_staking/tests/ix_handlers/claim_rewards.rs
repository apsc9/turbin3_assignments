use {
    anchor_lang::InstructionData,
    anchor_spl::associated_token,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_claim_rewards_ix(
    owner: &Address,
    config: &Address,
    asset: &Address,
    collection: &Address,
    update_authority: &Address,
    rewards_mint: &Address,
) -> Instruction {
    let program_id = pk(nft_staking::id());
    let system_program = pk(anchor_lang::system_program::ID);
    let mpl_core_program = pk(mpl_core::ID);
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());
    let associated_token_program = Address::from(associated_token::ID.to_bytes());

    let user_rewards_ata = Address::from(
        associated_token::get_associated_token_address(
            &anchor_lang::prelude::Pubkey::new_from_array(owner.to_bytes()),
            &anchor_lang::prelude::Pubkey::new_from_array(rewards_mint.to_bytes()),
        )
        .to_bytes(),
    );

    Instruction::new_with_bytes(
        program_id,
        &nft_staking::instruction::ClaimRewards {}.data(),
        vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*asset, false),
            AccountMeta::new(*collection, false),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new(*rewards_mint, false),
            AccountMeta::new(user_rewards_ata, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(associated_token_program, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(mpl_core_program, false),
        ],
    )
}
