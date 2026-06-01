use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_cancel_offer_ix(
    buyer: &Address,
    asset: &Address,
    offer: &Address,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let system_program = pk(anchor_lang::system_program::ID);

    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::CancelOffer {}.data(),
        vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new_readonly(*asset, false),
            AccountMeta::new(*offer, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    )
}
