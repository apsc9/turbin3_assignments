use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, TokenAccount, TransferChecked, transfer_checked, Mint, CloseAccount, close_account};
use anchor_spl::associated_token::AssociatedToken;
use crate::{ESCROW_SEED, Escrow};


// taker
// maker - system_account
// mint_a and mint_b
// taker_ata_a (which will receive the funds from vault)
// taker_ata_b (which will send the funds to the maker)
// maker_ata_b (which will receive the funds from the taker)
// escrow and vault accounts
// associated token program, token program and system program

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account[
        mint::token_program = token_program
    ]]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account[
        mint::token_program = token_program
    ]]
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account[
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    ]]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account[
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    ]]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account[
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    ]]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account[
        mut,
        close = maker,
        seeds = [ESCROW_SEED, escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker,
    ]]
    pub escrow: Box<Account<'info, Escrow>>,
    #[account[
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    ]]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info>Take<'info> {
    // transfer tokens from taker to maker
    pub fn deposit(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.key(), cpi_accounts);
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)

    }
    // withdraw tokens from vault to taker and also close the vault
    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            ESCROW_SEED,
            self.escrow.maker.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, &signer_seeds);
        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_a.decimals)?;

        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, &signer_seeds);
        close_account(cpi_context)
    }
}