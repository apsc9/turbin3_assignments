use {
    anchor_lang::InstructionData,
    solana_message::{Address, AccountMeta, Instruction},
};
use crate::pk;

pub fn create_make_offer_ix(
    buyer: &Address,
    asset: &Address,
    listing: &Address,
    offer: &Address,
    amount: u64,
) -> Instruction {
    let program_id = pk(nft_marketplace::id());
    let system_program = pk(anchor_lang::system_program::ID);

    Instruction::new_with_bytes(
        program_id,
        &nft_marketplace::instruction::MakeOffer { amount }.data(),
        vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new_readonly(*asset, false),
            AccountMeta::new_readonly(*listing, false),
            AccountMeta::new(*offer, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    )
}
