pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("F4uSdLrfQXJieQPi8R3u6MaeFVDbc5zNLeYEcMwn464y");

#[program]
pub mod dice_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.init(amount)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, seed: u128, roll: u8, amount: u64) -> Result<()> {
        ctx.accounts.create_bet(&ctx.bumps, seed, roll, amount)?;
        ctx.accounts.deposit(amount)
    }

    pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund_bet(&ctx.bumps)
    }

    pub fn submit_resolution(ctx: Context<SubmitResolution>, resolution: HouseResolution) -> Result<()> {
        ctx.accounts.submit_resolution(resolution)
    }

    pub fn resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
        ctx.accounts.resolve_bet(&ctx.bumps)
    }
}
