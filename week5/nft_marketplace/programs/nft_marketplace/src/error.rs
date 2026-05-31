use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("Name is too long")]
    NameTooLong,
    #[msg("Name cannot be empty")]
    NameEmpty,
    #[msg("Fee is greater than 100%. That is not a good idea")]
    InvalidFee,
    #[msg("Listing: NFT is not from allowed collection")]
    InvalidNftCollection,
    #[msg("Treasury Withdraw: Not an Admin")]
    NotAdmin,
    #[msg("Marketplace is currently Paused")]
    MarketplacePaused,
    #[msg("Overflow in fee calculations")]
    ArithmeticOverflow,
    #[msg("Insufficient treasury balance, put a lower amount")]
    InsufficientTreasuryBalance,
}