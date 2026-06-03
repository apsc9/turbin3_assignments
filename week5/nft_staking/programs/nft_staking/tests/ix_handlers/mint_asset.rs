use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_mint_asset_ix(
    user: &Address,
    asset: &Address,
    collection: &Address,
    update_authority: &Address,
    name: &str,
    uri: &str,
) -> Instruction {
    let program_id = pk(nft_staking::id());
    let system_program = pk(anchor_lang::system_program::ID);
    let mpl_core_program = pk(mpl_core::ID);

    Instruction::new_with_bytes(
        program_id,
        &nft_staking::instruction::MintAsset {
            name: name.to_string(),
            uri: uri.to_string(),
        }
        .data(),
        vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*asset, true),
            AccountMeta::new(*collection, false),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(mpl_core_program, false),
        ],
    )
}
