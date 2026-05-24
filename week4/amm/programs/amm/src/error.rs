use anchor_lang::prelude::*;
use constant_product_curve::CurveError;

#[error_code]
pub enum AmmError {
    #[msg("fee percentage can only be between 0 to 100(10000 bp)")]
    FeePercentError,
    #[msg("Default Error")]
    DefaultError,
    #[msg("Offer Expired")]
    OfferExpired,
    #[msg("This pool is locked")]
    PoolLocked,
    #[msg("Slippage Exceeded")]
    SlippageExceeded,
    #[msg("Overflow detected")]
    Overflow,
    #[msg("Underflow detected")]
    Underflow,
    #[msg("Invalid token")]
    InvalidToken,
    #[msg("Actual liquidkty is less than minimum")]
    LiquidityLessThanMinimum,
    #[msg("No liquidity in pool")]
    NoLiquidityInPool,
    #[msg("Bump Error")]
    BumpError,
    #[msg("Curve Error")]
    CurveError,
    #[msg("Fee is greater than 100%. This is not a very good deal")]
    InvalidFee,
    #[msg("Invalid Update Authority")]
    InvalidAuthority,
    #[msg("No update authority set")]
    NoAuthoritySet,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Invalid Precision")]
    InvalidPrecision,
    #[msg("Insufficient Balance")]
    InsufficientBalance,
    #[msg("Zero Balance")]
    ZeroBalance,
}

impl From<CurveError> for AmmError{
    fn from(error: CurveError) -> AmmError {
        match error {
            CurveError::InvalidPrecision => AmmError::InvalidPrecision,
            CurveError::Overflow => AmmError::Overflow,
            CurveError::Underflow => AmmError::Underflow,
            CurveError::InvalidFeeAmount => AmmError::InvalidFee,
            CurveError::InsufficientBalance => AmmError::InsufficientBalance,
            CurveError::ZeroBalance => AmmError::ZeroBalance,
            CurveError::SlippageLimitExceeded => AmmError::SlippageExceeded,
        }
    }
}
