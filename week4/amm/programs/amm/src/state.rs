use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,                    // seed to be able to create different pools/configs
    pub authority: Option<Pubkey>,    // in case we need to lock the config account
    pub mint_x: Pubkey,                 
    pub mint_y: Pubkey,
    pub fee: u16,                     // swap fees in basis points  
    pub locked: bool,                 // if the pool is locked
    pub config_bump: u8,              // bump seed for the config account
    pub lp_bump: u8,                  // bump seed for the LP token
}