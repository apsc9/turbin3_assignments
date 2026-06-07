use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use solana_instructions_sysvar::load_instruction_at_checked;

use crate::{
    error::DiceError,
    state::{Bet, HOUSE_EDGE_BASIS_POINTS},
};

/// The house commits the dice result by including a `submit_resolution`
/// instruction before `resolve_bet` in the same transaction.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct HouseResolution {
    pub bet_key: Pubkey,
    pub result_roll: u8,
    pub timestamp: i64,
    /// SHA-256 hash of (bet_serialized_data ++ result_roll)
    pub result_hash: [u8; 32],
}

#[derive(Accounts)]
pub struct SubmitResolution<'info> {
    #[account(mut)]
    pub house: Signer<'info>,
}

impl<'info> SubmitResolution<'info> {
    pub fn submit_resolution(&mut self, _resolution: HouseResolution) -> Result<()> {
        // No-op: this instruction exists solely so that `resolve_bet` can
        // introspect it from the instructions sysvar. The data is verified there.
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,

    #[account(mut)]
    /// CHECK: validated via bet.player constraint
    pub player: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        close = player,
        seeds = [b"bet", vault.key().as_ref(), bet.player.as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump
    )]
    pub bet: Account<'info, Bet>,

    /// CHECK: must be the instructions sysvar
    #[account(address = solana_sdk_ids::sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps) -> Result<()> {
        // Load the preceding instruction (index 0) from the same transaction.
        // The transaction layout needs to be [submit_resolution, resolve_bet]
        let ix = load_instruction_at_checked(0, &self.instructions_sysvar.to_account_info())
            .map_err(|_| DiceError::MissingResolutionInstruction)?;

        // Verify the preceding instruction was to our own program
        require_keys_eq!(ix.program_id, crate::ID, DiceError::InvalidResolutionProgram);

        // Deserialize our custom HouseResolution struct from the instruction data.
        // Anchor prefixes instruction data with an 8-byte discriminator.
        let ix_data = &ix.data;
        require!(ix_data.len() > 8, DiceError::InvalidResolutionData);

        let resolution = HouseResolution::deserialize(&mut &ix_data[8..])
            .map_err(|_| DiceError::InvalidResolutionData)?;

        // Verify the resolution targets this bet
        require_keys_eq!(
            resolution.bet_key,
            self.bet.key(),
            DiceError::ResolutionBetMismatch
        );

        // Verify the result_roll is within valid range (1-100)
        require!(
            resolution.result_roll >= 1 && resolution.result_roll <= 100,
            DiceError::InvalidResultRoll
        );

        // Verify the hash commitment: sha256(bet_data ++ result_roll)
        let mut hash_input = self.bet.to_slice();
        hash_input.push(resolution.result_roll);
        let expected_hash = solana_sha256_hasher::hash(&hash_input);
        require!(
            resolution.result_hash == expected_hash.to_bytes(),
            DiceError::InvalidResolutionHash
        );

        // Resolving the bet
        // Player wins if result_roll is <= bet.roll
        if resolution.result_roll <= self.bet.roll {
            // Calculate payout: (100 / roll) * amount, minus house edge
            let payout = (self.bet.amount as u128)
                .checked_mul(10000 - HOUSE_EDGE_BASIS_POINTS as u128)
                .ok_or(DiceError::Overflow)?
                .checked_mul(100)
                .ok_or(DiceError::Overflow)?
                .checked_div(self.bet.roll as u128)
                .ok_or(DiceError::Overflow)?
                .checked_div(10000)
                .ok_or(DiceError::Overflow)? as u64;

            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            let signer_seeds: &[&[&[u8]]] =
                &[&[b"vault", self.house.key.as_ref(), &[bumps.vault]]];

            let ctx = CpiContext::new_with_signer(
                self.system_program.key(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, payout)?;
        }

        Ok(())
    }
}
