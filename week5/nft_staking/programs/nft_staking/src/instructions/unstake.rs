use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, mint_to_checked, MintToChecked}};
use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::{UpdatePluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder},
    types::{UpdateAuthority, Attribute, Attributes, Plugin, PluginType, FreezeDelegate},
    fetch_plugin,
};
use crate::state::Config;
use crate::error::ErrorCode;

const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidOwner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ErrorCode::InvalidUpdateAuthority,
    )]
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: This account is not initialized and is used for signing purposes only, we verify that derives from the correct seeds
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds =[b"rewards_mint", config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = rewards_mint,
        associated_token::authority = owner,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the MPL Core Program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Unstake>) -> Result<()> {

     // We start by fetching the existing attributes(if any exist)
    let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes> (
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_,attrs,_)| attrs);

    require!(attributes_fetched.is_some(), ErrorCode::AssetNotStaked);

    let attributes = attributes_fetched.unwrap();

    // Prepare the Attributes list to update based on the existing attributes
    let mut attributes_list = Vec::with_capacity(attributes.attribute_list.len());

    // Additional auxiliary variables
    let current_timestamp = Clock::get()?.unix_timestamp;
    let mut staked_timestamp: i64;
    let mut reward_start: i64 = 0;

    for attribute in &attributes.attribute_list {
        if attribute.key == "staked" {
            require!(attribute.value == "true", ErrorCode::AssetNotStaked);
        }
        else if attribute.key == "staked_at" {
            staked_timestamp = attribute.value.parse::<i64>().map_err(|_| ErrorCode::InvalidTimestamp)?;

            // Freeze period check uses original stake time
            let staked_days = current_timestamp
                .checked_sub(staked_timestamp)
                .ok_or(ErrorCode::InvalidTimestamp)?
                .checked_div(SECONDS_PER_DAY)
                .ok_or(ErrorCode::InvalidTimestamp)?;
            require!(staked_days >= ctx.accounts.config.freeze_period as i64, ErrorCode::FreezePeriodNotElapsed);
        }
        else if attribute.key == "last_claimed" {
            // Reward calculation starts from last_claimed (avoids double rewards after claim)
            reward_start = attribute.value.parse::<i64>().map_err(|_| ErrorCode::InvalidTimestamp)?;
        }
        else {
            attributes_list.push(attribute.clone());
        }
    }

    // prepare the signer seeds for the update authority
    let collection_key = ctx.accounts.collection.key();
    let signer_seeds = &[
        b"update_authority",
        collection_key.as_ref(),
        &[ctx.bumps.update_authority]
    ];

    // Now we update the asset Attributes Plugin (with the existing attributes, including the Staking attributes with reset values)

    attributes_list.push(Attribute { 
        key: "staked".to_string(), 
        value: "false".to_string(), 
    });

    attributes_list.push(Attribute { 
        key: "staked_at".to_string(), 
        value: "0".to_string(),
    });

    UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
        .invoke_signed(&[signer_seeds])?;

    // And we Thaw the asset(update the FreezeDelegate Plugin to false)
    UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
        .invoke_signed(&[signer_seeds])?;

    // Decrement collection-level staked_count
    // The collection Attributes plugin was added during stake, so it must exist here
    let collection_attrs: Attributes = fetch_plugin::<BaseCollectionV1, Attributes>(
        &ctx.accounts.collection.to_account_info(),
        PluginType::Attributes,
    )
    .map(|(_, attrs, _)| attrs)
    .map_err(|_| ErrorCode::AssetNotStaked)?;

    let mut collection_attrs_list: Vec<Attribute> = Vec::new();
    let mut staked_count: u64 = 0;

    for attr in &collection_attrs.attribute_list {
        if attr.key == "staked_count" {
            staked_count = attr.value.parse::<u64>().unwrap_or(0);
        } else {
            collection_attrs_list.push(attr.clone());
        }
    }

    staked_count = staked_count.saturating_sub(1);

    collection_attrs_list.push(Attribute {
        key: "staked_count".to_string(),
        value: staked_count.to_string(),
    });

    UpdateCollectionPluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: collection_attrs_list }))
        .invoke_signed(&[signer_seeds])?;

    // Mint remaining rewards (from last_claimed to now), skip if 0 (e.g. unstake right after claim)

    let claimable_days = current_timestamp
        .checked_sub(reward_start)
        .ok_or(ErrorCode::InvalidTimestamp)?
        .checked_div(SECONDS_PER_DAY)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    if claimable_days > 0 {
        let amount = (claimable_days as u64)
                .checked_mul(ctx.accounts.config.rewards_bps as u64)
                .ok_or(ErrorCode::InvalidRewardsBps)?
                .checked_mul(10u64.pow(ctx.accounts.rewards_mint.decimals as u32))
                .ok_or(ErrorCode::InvalidRewardsBps)?
                .checked_div(10000u64)
                .ok_or(ErrorCode::InvalidRewardsBps)?;

        let config_seeds = &[
            b"config",
            collection_key.as_ref(),
            &[ctx.accounts.config.bump]
        ];

        mint_to_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintToChecked {
                    mint: ctx.accounts.rewards_mint.to_account_info(),
                    to: ctx.accounts.user_rewards_ata.to_account_info(),
                    authority: ctx.accounts.config.to_account_info(),
                },
                &[config_seeds],
            ),
            amount,
            ctx.accounts.rewards_mint.decimals,
        )?;
    }

    Ok(())


}
