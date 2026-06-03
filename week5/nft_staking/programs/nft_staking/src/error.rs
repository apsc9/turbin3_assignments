use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Asset owner")]
    InvalidOwner,
    #[msg("Invalid Update authority")]
    InvalidUpdateAuthority,
    #[msg("Asset already staked")]
    AlreadyStaked,
    #[msg("Asset not staked")]
    AssetNotStaked,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Freeze period not elapsed")]
    FreezePeriodNotElapsed,
    #[msg("Invalid rewards bps")]
    InvalidRewardsBps,
    #[msg("No rewards to claim")]
    NoRewardsToClaim,
    #[msg("Arithmetic overflow")]
    Overflow,
}
