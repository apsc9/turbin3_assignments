use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_stake_ix(
    owner: &Address,
    config: &Address,
    asset: &Address,
    collection: &Address,
    update_authority: &Address,
) -> Instruction {
    let program_id = pk(nft_staking::id());
    let system_program = pk(anchor_lang::system_program::ID);
    let mpl_core_program = pk(mpl_core::ID);

    Instruction::new_with_bytes(
        program_id,
        &nft_staking::instruction::Stake {}.data(),
        vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*asset, false),
            AccountMeta::new(*collection, false),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(mpl_core_program, false),
        ],
    )
}
