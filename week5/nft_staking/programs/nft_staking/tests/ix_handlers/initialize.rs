use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_initialize_ix(
    admin: &Address,
    config: &Address,
    collection: &Address,
    update_authority: &Address,
    rewards_mint: &Address,
    rewards_bps: u16,
    freeze_period: u16,
) -> Instruction {
    let program_id = pk(nft_staking::id());
    let system_program = pk(anchor_lang::system_program::ID);
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());

    Instruction::new_with_bytes(
        program_id,
        &nft_staking::instruction::Initialize {
            rewards_bps,
            freeze_period,
        }
        .data(),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(*collection, false),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new(*rewards_mint, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}
