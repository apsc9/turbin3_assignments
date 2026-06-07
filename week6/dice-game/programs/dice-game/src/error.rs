use anchor_lang::prelude::*;

#[error_code]
pub enum DiceError {
    #[msg("Bet needs to be greater than the min value")]
    MinimumBet,
    #[msg("Roll needs to be greater than the min value")]
    MinimumRoll,
    #[msg("Roll needs to be less than the max vallue")]
    MaximumRoll,
    #[msg("Overflow Error")]
    Overflow,
    #[msg("Time out not reached")]
    TimeoutNotReached,
    #[msg("Missing resolution instruction in transaction")]
    MissingResolutionInstruction,
    #[msg("Resolution instruction must be from this program")]
    InvalidResolutionProgram,
    #[msg("Invalid resolution instruction data")]
    InvalidResolutionData,
    #[msg("Resolution bet key does not match")]
    ResolutionBetMismatch,
    #[msg("Result roll must be between 1 and 100")]
    InvalidResultRoll,
    #[msg("Resolution hash does not match computed hash")]
    InvalidResolutionHash,
}
