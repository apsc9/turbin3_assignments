use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use crate::*;

#[derive(Accounts)]
pub struct WithdrawFee<'info>{
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
        has_one = admin
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

impl<'info> WithdrawFee<'info> {
    pub fn withdraw_fee(&mut self, amount: u64) -> Result<()> {

        let rent = Rent::get()?;
        let min_balance = rent.minimum_balance(0); // 0 bytes data for SystemAccount
        let available = self.treasury.lamports().checked_sub(min_balance).unwrap();

        require!(amount <= available, MarketplaceError::InsufficientTreasuryBalance);

        let marketplace_key = self.marketplace.key();
        let seeds: &[&[u8]] = &[b"treasury", marketplace_key.as_ref(), &[self.marketplace.treasury_bump]];
        let signer_seeds = &[seeds];

        transfer(CpiContext::new_with_signer(
            self.system_program.to_account_info(), 
            Transfer { 
                from: self.treasury.to_account_info(), 
                to: self.admin.to_account_info(),
            }, 
            signer_seeds,
        ), 
        amount)?;

        Ok(())

    }
}



// challenge 1 :
// make it work with both sol and some other mint token like usdc
// fake a usdc mint and make it available for both sol and usdc 

// create new instruction: make-offer : which will basically give counter offer
// delist : just to have your nft back 
// make it possible to use spl token and sol both for these transactions/