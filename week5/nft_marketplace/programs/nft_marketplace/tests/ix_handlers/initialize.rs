use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_initialize_ix(
    admin: &Address,
    marketplace: &Address,
    treasury: &Address,
    rewards_mint: &Address,
    name: &str,
    fee: u16,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let system_program = pk(anchor_lang::system_program::ID);
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());

    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::Initialize {
            name: name.to_string(),
            fee,
        }
        .data(),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(*marketplace, false),
            AccountMeta::new_readonly(*treasury, false),
            AccountMeta::new(*rewards_mint, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}
