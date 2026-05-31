use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::{associated_token::AssociatedToken, token::{MintTo, mint_to}, token_interface::{Mint, TokenAccount, TokenInterface}};
use mpl_core::{ID as MPL_CORE_ID, collection, instructions::TransferV1CpiBuilder};

use crate::*;

#[derive(Accounts)]
pub struct Buy<'info> {

    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: 
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: validate during the cpi transfer by mpl-core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        close = maker,
        seeds = [b"listing", listing.asset.key().as_ref()],
        bump = listing.bump,
        has_one = maker,
        has_one = asset,
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_rewards_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: 
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>
}

impl<'info> Buy<'info> {
    pub fn send_sol(&mut self) -> Result<()> {

        let price = self.listing.price;
        let fee = (price as u128)
            .checked_mul(self.marketplace.fee as u128)
            .unwrap()
            .checked_div(10_000)
            .unwrap() as u64;

        let maker_amount = price.checked_sub(fee).unwrap();

        transfer(CpiContext::new(
            self.system_program.to_account_info(), 
            Transfer{
                from: self.taker.to_account_info(),
                to: self.maker.to_account_info(),
            }), 
            maker_amount
        )?;

        transfer(CpiContext::new(
            self.system_program.to_account_info(), 
            Transfer{
                from: self.taker.to_account_info().to_account_info(),
                to: self.treasury.to_account_info(),
            }), 
            fee
        )?;
        Ok(())
    }

    pub fn receive_nft(&mut self) -> Result<()> {
        let asset_key = self.asset.key();
        let bump = self.listing.bump;
        let seeds : &[&[u8]] = &[b"listing", asset_key.as_ref(), &[bump]];
        let signer_seeds = &[seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.taker.to_account_info())
            .authority(Some(&self.listing.to_account_info()))
            .new_owner(&self.taker.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn receive_rewards(&mut self) -> Result<()> {
        let seeds: &[&[u8]] = &[
            b"marketplace",
            self.marketplace.name.as_str().as_bytes(),
            &[self.marketplace.bump],
        ];
        let signer_seeds = &[seeds];

        mint_to(CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            MintTo{
                mint: self.rewards_mint.to_account_info(),
                to: self.taker_rewards_ata.to_account_info(),
                authority: self.marketplace.to_account_info(),
            }, 
            signer_seeds), 
            1
        )?;
        Ok(())
    }
}