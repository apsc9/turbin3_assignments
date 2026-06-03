use {
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Address, Instruction, Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

pub const REWARDS_BPS: u16 = 1000; // 10% per day
pub const FREEZE_PERIOD: u16 = 1;  // 1 day minimum

pub fn pk(old: anchor_lang::prelude::Pubkey) -> Address {
    Address::from(old.to_bytes())
}

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let staking_bytes = include_bytes!("../../../target/deploy/nft_staking.so");
    svm.add_program(pk(nft_staking::id()), staking_bytes).unwrap();

    let mpl_core_bytes = include_bytes!("fixtures/mpl_core.so");
    svm.add_program(pk(mpl_core::ID), mpl_core_bytes).unwrap();

    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

    (svm, admin)
}

fn staking_pdas(collection: &Address) -> (Address, Address, Address) {
    let program_id = pk(nft_staking::id());

    let (config, _) = Address::find_program_address(
        &[b"config", collection.as_ref()],
        &program_id,
    );
    let (update_authority, _) = Address::find_program_address(
        &[b"update_authority", collection.as_ref()],
        &program_id,
    );
    let (rewards_mint, _) = Address::find_program_address(
        &[b"rewards_mint", config.as_ref()],
        &program_id,
    );
    (config, update_authority, rewards_mint)
}

fn warp_days(svm: &mut LiteSVM, days: u64) {
    let mut clock: solana_clock::Clock = svm.get_sysvar();
    clock.unix_timestamp += (days * 86400) as i64;
    svm.set_sysvar(&clock);
}

// Shared setup: creates collection + config + mints an asset, returns everything needed for staking tests
fn setup_staked_env() -> (LiteSVM, Keypair, Keypair, Address, Address, Address, Address) {
    let (mut svm, admin) = setup();

    // Create collection via the program (sets update_authority to PDA)
    let collection = Keypair::new();
    let (config, update_authority, rewards_mint) = staking_pdas(&collection.pubkey());

    let coll_ix = create_create_collection_ix(
        &admin.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test Collection",
        "https://test.com/collection.json",
    );
    send(&mut svm, &[coll_ix], &admin, &[&admin, &collection]).unwrap();

    // Initialize staking config
    let init_ix = create_initialize_ix(
        &admin.pubkey(),
        &config,
        &collection.pubkey(),
        &update_authority,
        &rewards_mint,
        REWARDS_BPS,
        FREEZE_PERIOD,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    // Mint an NFT
    let asset = Keypair::new();
    let mint_ix = create_mint_asset_ix(
        &admin.pubkey(),
        &asset.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test NFT",
        "https://test.com/nft.json",
    );
    send(&mut svm, &[mint_ix], &admin, &[&admin, &asset]).unwrap();

    (svm, admin, asset, collection.pubkey(), config, update_authority, rewards_mint)
}

// ============================================================
// Tests
// ============================================================

#[test]
fn test_create_collection() {
    let (mut svm, admin) = setup();

    let collection = Keypair::new();
    let (_, update_authority, _) = staking_pdas(&collection.pubkey());

    let ix = create_create_collection_ix(
        &admin.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test Collection",
        "https://test.com/collection.json",
    );
    let res = send(&mut svm, &[ix], &admin, &[&admin, &collection]);
    assert!(res.is_ok(), "create_collection failed: {:?}", res.err());
}

#[test]
fn test_initialize() {
    let (mut svm, admin) = setup();

    let collection = Keypair::new();
    let (config, update_authority, rewards_mint) = staking_pdas(&collection.pubkey());

    // Must create collection first (initialize reads it)
    let coll_ix = create_create_collection_ix(
        &admin.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test Collection",
        "https://test.com/collection.json",
    );
    send(&mut svm, &[coll_ix], &admin, &[&admin, &collection]).unwrap();

    let ix = create_initialize_ix(
        &admin.pubkey(),
        &config,
        &collection.pubkey(),
        &update_authority,
        &rewards_mint,
        REWARDS_BPS,
        FREEZE_PERIOD,
    );
    let res = send(&mut svm, &[ix], &admin, &[&admin]);
    assert!(res.is_ok(), "initialize failed: {:?}", res.err());
}

#[test]
fn test_mint_asset() {
    let (mut svm, admin) = setup();

    let collection = Keypair::new();
    let (_, update_authority, _) = staking_pdas(&collection.pubkey());

    let coll_ix = create_create_collection_ix(
        &admin.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test Collection",
        "https://test.com/collection.json",
    );
    send(&mut svm, &[coll_ix], &admin, &[&admin, &collection]).unwrap();

    let asset = Keypair::new();
    let ix = create_mint_asset_ix(
        &admin.pubkey(),
        &asset.pubkey(),
        &collection.pubkey(),
        &update_authority,
        "Test NFT",
        "https://test.com/nft.json",
    );
    let res = send(&mut svm, &[ix], &admin, &[&admin, &asset]);
    assert!(res.is_ok(), "mint_asset failed: {:?}", res.err());
}

#[test]
fn test_stake() {
    let (mut svm, admin, asset, collection, config, update_authority, _) = setup_staked_env();

    let ix = create_stake_ix(
        &admin.pubkey(),
        &config,
        &asset.pubkey(),
        &collection,
        &update_authority,
    );
    let res = send(&mut svm, &[ix], &admin, &[&admin]);
    assert!(res.is_ok(), "stake failed: {:?}", res.err());
}

#[test]
fn test_stake_already_staked_fails() {
    let (mut svm, admin, asset, collection, config, update_authority, _) = setup_staked_env();

    let ix = create_stake_ix(
        &admin.pubkey(),
        &config,
        &asset.pubkey(),
        &collection,
        &update_authority,
    );
    send(&mut svm, &[ix.clone()], &admin, &[&admin]).unwrap();

    // Staking again should fail
    let res = send(&mut svm, &[ix], &admin, &[&admin]);
    assert!(res.is_err(), "double stake should fail");
}

#[test]
fn test_unstake() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    // Stake
    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    // Warp past freeze period
    warp_days(&mut svm, 2);

    // Unstake
    let unstake_ix = create_unstake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    let res = send(&mut svm, &[unstake_ix], &admin, &[&admin]);
    assert!(res.is_ok(), "unstake failed: {:?}", res.err());
}

#[test]
fn test_unstake_before_freeze_period_fails() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    // Don't warp time — freeze period not elapsed
    let unstake_ix = create_unstake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    let res = send(&mut svm, &[unstake_ix], &admin, &[&admin]);
    assert!(res.is_err(), "unstake before freeze period should fail");
}

#[test]
fn test_claim_rewards() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    // Stake
    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    // Warp 3 days
    warp_days(&mut svm, 3);

    // Claim rewards (NFT stays staked)
    let claim_ix = create_claim_rewards_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    let res = send(&mut svm, &[claim_ix], &admin, &[&admin]);
    assert!(res.is_ok(), "claim_rewards failed: {:?}", res.err());
}

#[test]
fn test_claim_rewards_no_time_elapsed_fails() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    // No time warp — should fail with NoRewardsToClaim
    let claim_ix = create_claim_rewards_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    let res = send(&mut svm, &[claim_ix], &admin, &[&admin]);
    assert!(res.is_err(), "claim with no elapsed time should fail");
}

#[test]
fn test_claim_then_unstake() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    // Stake
    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    // Warp past freeze period
    warp_days(&mut svm, 3);

    // Claim rewards first
    let claim_ix = create_claim_rewards_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    send(&mut svm, &[claim_ix], &admin, &[&admin]).unwrap();

    // Unstake immediately after claiming — should succeed (freeze checks staked_at, not last_claimed)
    let unstake_ix = create_unstake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    let res = send(&mut svm, &[unstake_ix], &admin, &[&admin]);
    assert!(res.is_ok(), "unstake after claim failed: {:?}", res.err());
}

#[test]
fn test_stake_unstake_restake() {
    let (mut svm, admin, asset, collection, config, update_authority, rewards_mint) =
        setup_staked_env();

    // Stake
    let stake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    send(&mut svm, &[stake_ix], &admin, &[&admin]).unwrap();

    warp_days(&mut svm, 2);

    // Unstake
    let unstake_ix = create_unstake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority, &rewards_mint,
    );
    send(&mut svm, &[unstake_ix], &admin, &[&admin]).unwrap();

    // Re-stake — verifies attributes reset correctly and FreezeDelegate can be re-added
    let restake_ix = create_stake_ix(
        &admin.pubkey(), &config, &asset.pubkey(), &collection, &update_authority,
    );
    let res = send(&mut svm, &[restake_ix], &admin, &[&admin]);
    assert!(res.is_ok(), "restake failed: {:?}", res.err());
}
