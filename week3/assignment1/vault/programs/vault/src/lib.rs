use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod constants;

pub use state::*;
pub use instructions::*;

declare_id!("9nz5PoEPJoZaghNwSYAheVW8xgcZUuxjHNn6vZY636Ja");

#[program]
pub mod vault {
    use super::*;

    // intiialize
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    // deposit
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    // withdraw
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    // close the vault
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}
