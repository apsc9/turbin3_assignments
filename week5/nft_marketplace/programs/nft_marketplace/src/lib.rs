pub mod instructions;
pub mod state;
pub mod error;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use error::*;

declare_id!("CiQM2wDGJBymguyD874w8uSPSrvFK8jnhnTAbQShKfTQ");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, fee: u16) -> Result<()> {
        ctx.accounts.init(name, fee, &ctx.bumps)
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.receive_nft()?;
        ctx.accounts.receive_rewards()
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.delist_nft()
    }

    pub fn withdraw(ctx: Context<WithdrawFee>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw_fee(amount)
    }

    pub fn buy_with_token(ctx: Context<BuyWithToken>) -> Result<()> {
        ctx.accounts.send_token()?;
        ctx.accounts.receive_nft()?;
        ctx.accounts.receive_rewards()
    }
}
