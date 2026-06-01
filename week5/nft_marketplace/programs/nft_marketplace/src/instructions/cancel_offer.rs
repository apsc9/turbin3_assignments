use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::*;

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Only needed to derive offer PDA
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer", asset.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        has_one = buyer,
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

impl<'info> CancelOffer<'info> {
    pub fn cancel_offer(&mut self) -> Result<()> {
        let asset_key = self.asset.key();
        let buyer_key = self.buyer.key();
        let bump = self.offer.bump;
        let seeds: &[&[u8]] = &[b"offer", asset_key.as_ref(), buyer_key.as_ref(), &[bump]];
        let signer_seeds = &[seeds];

        let escrowed_amount = self.offer.amount;

        // transfer escrowed sol from Offer PDA to buyer
        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(), 
                Transfer { 
                    from: self.offer.to_account_info(), 
                    to: self.buyer.to_account_info(),
                }, 
                signer_seeds
            ), 
            escrowed_amount,
        )?;

        Ok(())
    }
}