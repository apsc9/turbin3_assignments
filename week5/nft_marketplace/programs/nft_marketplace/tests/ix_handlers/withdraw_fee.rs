use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_withdraw_fee_ix(
    admin: &Address,
    marketplace: &Address,
    treasury: &Address,
    amount: u64,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let system_program = pk(anchor_lang::system_program::ID);

    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::Withdraw { amount }.data(),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new_readonly(*marketplace, false),
            AccountMeta::new(*treasury, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    )
}
