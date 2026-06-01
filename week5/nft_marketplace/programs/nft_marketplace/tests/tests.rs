use {
    anchor_lang::InstructionData,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Address, Instruction, Message, AccountMeta, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

pub const MARKETPLACE_NAME: &str = "test_market";
pub const MARKETPLACE_FEE: u16 = 500; // 5%
pub const LISTING_PRICE: u64 = 1_000_000_000; // 1 SOL

// Convert anchor v2 Pubkey → v4 Pubkey (= Address)
// Needed because anchor 0.32.1 uses solana-program v2 types,
// while litesvm/message/transaction use solana-address v2.6 types.
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

    let marketplace_bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    svm.add_program(pk(nft_marketplace::id()), marketplace_bytes).unwrap();

    let mpl_core_bytes = include_bytes!("fixtures/mpl_core.so");
    svm.add_program(pk(mpl_core::ID), mpl_core_bytes).unwrap();

    let admin = Keypair::new();
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

    (svm, admin)
}

pub fn marketplace_pdas(name: &str) -> (Address, Address, Address) {
    let program_id = pk(nft_marketplace::id());
    let (marketplace, _) = Address::find_program_address(
        &[b"marketplace", name.as_bytes()],
        &program_id,
    );
    let (treasury, _) = Address::find_program_address(
        &[b"treasury", marketplace.as_ref()],
        &program_id,
    );
    let (rewards_mint, _) = Address::find_program_address(
        &[b"rewards", marketplace.as_ref()],
        &program_id,
    );
    (marketplace, treasury, rewards_mint)
}

pub fn listing_pda(asset: &Address) -> Address {
    Address::find_program_address(
        &[b"listing", asset.as_ref()],
        &pk(nft_marketplace::id()),
    ).0
}

pub fn offer_pda(asset: &Address, buyer: &Address) -> Address {
    Address::find_program_address(
        &[b"offer", asset.as_ref(), buyer.as_ref()],
        &pk(nft_marketplace::id()),
    ).0
}

// --- mpl-core helpers ---
// These build raw mpl-core instructions to create collections and assets in LiteSVM.
// We convert the v2 Instruction returned by mpl-core into v3 Instruction.

fn convert_ix(ix: anchor_lang::solana_program::instruction::Instruction) -> Instruction {
    Instruction::new_with_bytes(
        Address::from(ix.program_id.to_bytes()),
        &ix.data,
        ix.accounts
            .into_iter()
            .map(|a| {
                let pubkey = Address::from(a.pubkey.to_bytes());
                if a.is_writable {
                    AccountMeta::new(pubkey, a.is_signer)
                } else {
                    AccountMeta::new_readonly(pubkey, a.is_signer)
                }
            })
            .collect(),
    )
}

pub fn create_collection_ix(collection: &Keypair, payer: &Address) -> Instruction {
    let payer_v2 = anchor_lang::prelude::Pubkey::new_from_array(payer.to_bytes());
    let ix = mpl_core::instructions::CreateCollectionV1 {
        collection: anchor_lang::prelude::Pubkey::new_from_array(collection.pubkey().to_bytes()),
        update_authority: Some(payer_v2),
        payer: payer_v2,
        system_program: anchor_lang::system_program::ID,
    }
    .instruction(mpl_core::instructions::CreateCollectionV1InstructionArgs {
        name: "Test Collection".to_string(),
        uri: "https://test.com/collection.json".to_string(),
        plugins: None,
    });
    convert_ix(ix)
}

pub fn create_asset_ix(
    asset: &Keypair,
    collection: Option<&Address>,
    payer: &Address,
    owner: &Address,
) -> Instruction {
    let payer_v2 = anchor_lang::prelude::Pubkey::new_from_array(payer.to_bytes());
    let ix = mpl_core::instructions::CreateV1 {
        asset: anchor_lang::prelude::Pubkey::new_from_array(asset.pubkey().to_bytes()),
        collection: collection.map(|c| anchor_lang::prelude::Pubkey::new_from_array(c.to_bytes())),
        authority: Some(payer_v2),
        payer: payer_v2,
        owner: Some(anchor_lang::prelude::Pubkey::new_from_array(owner.to_bytes())),
        update_authority: None,
        system_program: anchor_lang::system_program::ID,
        log_wrapper: None,
    }
    .instruction(mpl_core::instructions::CreateV1InstructionArgs {
        data_state: mpl_core::types::DataState::AccountState,
        name: "Test NFT".to_string(),
        uri: "https://test.com/nft.json".to_string(),
        plugins: None,
    });
    convert_ix(ix)
}

// --- SPL token helpers ---
// For buy_with_token test: need to create a payment mint and fund taker with tokens

fn create_spl_mint_ix(mint: &Keypair, payer: &Address, decimals: u8) -> Vec<Instruction> {
    let mint_addr = mint.pubkey();
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());
    let system_program = pk(anchor_lang::system_program::ID);
    let rent_sysvar = Address::from(anchor_lang::solana_program::sysvar::rent::ID.to_bytes());

    let space = 82u64; // SPL Mint account size
    let create_account_ix = Instruction::new_with_bytes(
        system_program,
        &anchor_lang::solana_program::system_instruction::create_account(
            &anchor_lang::prelude::Pubkey::new_from_array(payer.to_bytes()),
            &anchor_lang::prelude::Pubkey::new_from_array(mint_addr.to_bytes()),
            1_461_600, // rent for 82 bytes
            space,
            &anchor_lang::prelude::Pubkey::new_from_array(token_program.to_bytes()),
        ).data,
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(mint_addr, true),
            AccountMeta::new_readonly(system_program, false),
        ],
    );

    let mut init_data = vec![0u8]; // InitializeMint instruction index = 0
    init_data.extend_from_slice(&[decimals]);
    init_data.extend_from_slice(&payer.to_bytes()); // mint authority
    init_data.push(1); // COption::Some for freeze authority
    init_data.extend_from_slice(&payer.to_bytes()); // freeze authority

    let init_mint_ix = Instruction::new_with_bytes(
        token_program,
        &init_data,
        vec![
            AccountMeta::new(mint_addr, false),
            AccountMeta::new_readonly(rent_sysvar, false),
        ],
    );

    vec![create_account_ix, init_mint_ix]
}

fn mint_tokens_ix(
    mint: &Address,
    dest_ata: &Address,
    authority: &Address,
    amount: u64,
) -> Instruction {
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());

    let mut data = vec![7u8]; // MintTo instruction index = 7
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction::new_with_bytes(
        token_program,
        &data,
        vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*dest_ata, false),
            AccountMeta::new_readonly(*authority, true),
        ],
    )
}

fn create_ata_ix(payer: &Address, owner: &Address, mint: &Address) -> Instruction {
    let associated_token_program = Address::from(anchor_spl::associated_token::ID.to_bytes());
    let token_program = Address::from(anchor_spl::token::ID.to_bytes());
    let system_program = pk(anchor_lang::system_program::ID);

    let ata = Address::from(
        anchor_spl::associated_token::get_associated_token_address(
            &anchor_lang::prelude::Pubkey::new_from_array(owner.to_bytes()),
            &anchor_lang::prelude::Pubkey::new_from_array(mint.to_bytes()),
        ).to_bytes(),
    );

    Instruction::new_with_bytes(
        associated_token_program,
        &[],
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}

fn get_ata(owner: &Address, mint: &Address) -> Address {
    Address::from(
        anchor_spl::associated_token::get_associated_token_address(
            &anchor_lang::prelude::Pubkey::new_from_array(owner.to_bytes()),
            &anchor_lang::prelude::Pubkey::new_from_array(mint.to_bytes()),
        ).to_bytes(),
    )
}

// ============================================================
// Tests
// ============================================================

#[test]
fn test_initialize() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    let ix = create_initialize_ix(
        &admin.pubkey(),   
        &marketplace,      
        &treasury,         
        &rewards_mint,     
        MARKETPLACE_NAME,  // "test_market" — used in PDA seeds
        MARKETPLACE_FEE,   // 500 = 5% in basis points
    );
    let res = send(&mut svm, &[ix], &admin, &[&admin]);
    assert!(res.is_ok(), "initialize failed: {:?}", res.err());
}

#[test]
fn test_list() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    // Initialize marketplace first
    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    // create mpl core collection
    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    // Create mpl-core asset (the NFT)
    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset,
        Some(&collection.pubkey()),  // belongs to this collection
        &maker.pubkey(),             // payer
        &maker.pubkey(),             // owner
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    let listing = listing_pda(&asset.pubkey());

    let list_ix = create_list_ix(
        &maker.pubkey(),             
        &asset.pubkey(),             
        Some(&collection.pubkey()),  
        &listing,                    
        None,                        
        LISTING_PRICE,           
    );

    let res = send(&mut svm, &[list_ix], &maker, &[&maker]);
    assert!(res.is_ok(), "list failed: {:?}", res.err());
}

#[test]
fn test_buy() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    // Initialize marketplace 
    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset, Some(&collection.pubkey()),
        &maker.pubkey(), &maker.pubkey(),
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    let listing = listing_pda(&asset.pubkey());
    let list_ix = create_list_ix(
        &maker.pubkey(), &asset.pubkey(), Some(&collection.pubkey()),
        &listing, None, LISTING_PRICE,
    );
    send(&mut svm, &[list_ix], &maker, &[&maker]).unwrap();

    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 5_000_000_000).unwrap();

    // Build buy instruction
    let buy_ix = create_buy_ix(
        &taker.pubkey(),             
        &maker.pubkey(),             
        &asset.pubkey(),             
        Some(&collection.pubkey()),  
        &marketplace,                
        &listing,                    
        &treasury,                   
        &rewards_mint,               
    );

    let res = send(&mut svm, &[buy_ix], &taker, &[&taker]);
    assert!(res.is_ok(), "buy failed: {:?}", res.err());
}

#[test]
fn test_delist() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    // Initialize marketplace 
    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    // Maker creates collection + asset + lists ---
    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset, Some(&collection.pubkey()),
        &maker.pubkey(), &maker.pubkey(),
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    let listing = listing_pda(&asset.pubkey());
    let list_ix = create_list_ix(
        &maker.pubkey(), &asset.pubkey(), Some(&collection.pubkey()),
        &listing, None, LISTING_PRICE,
    );
    send(&mut svm, &[list_ix], &maker, &[&maker]).unwrap();

    let delist_ix = create_delist_ix(
        &maker.pubkey(),             
        &asset.pubkey(),             
        Some(&collection.pubkey()),  
        &listing,                    
    );

    let res = send(&mut svm, &[delist_ix], &maker, &[&maker]);
    assert!(res.is_ok(), "delist failed: {:?}", res.err());
}

#[test]
fn test_buy_with_token() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

   
    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    // Create payment mint (like USDC) 
    // This is the SPL token buyers will pay with
    // Keypair because it's a new account being created
    let payment_mint = Keypair::new();
    let mint_ixs = create_spl_mint_ix(&payment_mint, &admin.pubkey(), 6);
    send(&mut svm, &mint_ixs, &admin, &[&admin, &payment_mint]).unwrap();

    // Maker creates collection + asset 
    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset, Some(&collection.pubkey()),
        &maker.pubkey(), &maker.pubkey(),
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    // List with payment_mint (NOT None like before) ---
    // This makes listing.payment_mint = Some(payment_mint)
    // which satisfies BuyWithToken's constraint: payment_mint.is_some()
    let listing = listing_pda(&asset.pubkey());
    let list_ix = create_list_ix(
        &maker.pubkey(), &asset.pubkey(), Some(&collection.pubkey()),
        &listing,
        Some(&payment_mint.pubkey()),  // ← key difference from test_buy
        LISTING_PRICE,
    );
    send(&mut svm, &[list_ix], &maker, &[&maker]).unwrap();

    // Create taker + fund with tokens ---
    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 5_000_000_000).unwrap();

    // Create taker's ATA for payment_mint
    let taker_ata_ix = create_ata_ix(
        &taker.pubkey(), &taker.pubkey(), &payment_mint.pubkey(),
    );
    send(&mut svm, &[taker_ata_ix], &taker, &[&taker]).unwrap();

    // Mint tokens to taker's ATA
    // admin is mint authority (set during create_spl_mint_ix)
    let taker_ata = get_ata(&taker.pubkey(), &payment_mint.pubkey());
    let mint_to_ix = mint_tokens_ix(
        &payment_mint.pubkey(),
        &taker_ata,
        &admin.pubkey(),   // mint authority = admin
        2_000_000_000,     // 2000 tokens (6 decimals)
    );
    send(&mut svm, &[mint_to_ix], &admin, &[&admin]).unwrap();

    let buy_ix = create_buy_with_token_ix(
        &taker.pubkey(),             
        &maker.pubkey(),             
        &asset.pubkey(),             
        Some(&collection.pubkey()),  
        &marketplace,                
        &listing,                    
        &payment_mint.pubkey(),      
        &taker_ata,                 
        &rewards_mint,               
    );

    let res = send(&mut svm, &[buy_ix], &taker, &[&taker]);
    assert!(res.is_ok(), "buy_with_token failed: {:?}", res.err());
}

#[test]
fn test_withdraw_fee() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset, Some(&collection.pubkey()),
        &maker.pubkey(), &maker.pubkey(),
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    let listing = listing_pda(&asset.pubkey());
    let list_ix = create_list_ix(
        &maker.pubkey(), &asset.pubkey(), Some(&collection.pubkey()),
        &listing, None, LISTING_PRICE,
    );
    send(&mut svm, &[list_ix], &maker, &[&maker]).unwrap();

    // Taker buys — this puts fees into treasury ---
    // 5% of 1 SOL = 50_000_000 lamports go to treasury
    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 5_000_000_000).unwrap();
    let buy_ix = create_buy_ix(
        &taker.pubkey(), &maker.pubkey(), &asset.pubkey(),
        Some(&collection.pubkey()), &marketplace, &listing,
        &treasury, &rewards_mint,
    );
    send(&mut svm, &[buy_ix], &taker, &[&taker]).unwrap();

    // Treasury has 50_000_000 lamports from the buy fee
    // Withdraw 10_000_000 (must be <= available after rent)
    let withdraw_ix = create_withdraw_fee_ix(
        &admin.pubkey(),   // admin: must match marketplace.admin (has_one)
        &marketplace,      // marketplace: read-only, provides admin check + seeds
        &treasury,         // treasury: PDA, sends lamports
        10_000_000,        // amount: 0.01 SOL (well under the 0.05 SOL available)
    );

    // Only admin signs — treasury PDA signs internally via invoke_signed
    let res = send(&mut svm, &[withdraw_ix], &admin, &[&admin]);
    assert!(res.is_ok(), "withdraw_fee failed: {:?}", res.err());
}

#[test]
fn test_make_offer() {
    let (mut svm, admin) = setup();
    let (marketplace, treasury, rewards_mint) = marketplace_pdas(MARKETPLACE_NAME);

    let init_ix = create_initialize_ix(
        &admin.pubkey(), &marketplace, &treasury, &rewards_mint,
        MARKETPLACE_NAME, MARKETPLACE_FEE,
    );
    send(&mut svm, &[init_ix], &admin, &[&admin]).unwrap();

    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 5_000_000_000).unwrap();

    let collection = Keypair::new();
    let coll_ix = create_collection_ix(&collection, &maker.pubkey());
    send(&mut svm, &[coll_ix], &maker, &[&maker, &collection]).unwrap();

    let asset = Keypair::new();
    let asset_ix = create_asset_ix(
        &asset, Some(&collection.pubkey()),
        &maker.pubkey(), &maker.pubkey(),
    );
    send(&mut svm, &[asset_ix], &maker, &[&maker, &asset]).unwrap();

    let listing = listing_pda(&asset.pubkey());
    let list_ix = create_list_ix(
        &maker.pubkey(), &asset.pubkey(), Some(&collection.pubkey()),
        &listing, None, LISTING_PRICE,
    );
    send(&mut svm, &[list_ix], &maker, &[&maker]).unwrap();

    // Now test MakeOffer
    // Buyer offers 0.8 SOL (below listing price of 1 SOL — it's a counter-offer)
    let buyer = Keypair::new();
    svm.airdrop(&buyer.pubkey(), 5_000_000_000).unwrap();

    // Derive offer PDA — seeds = [b"offer", asset, buyer]
    let offer = offer_pda(&asset.pubkey(), &buyer.pubkey());
    let offer_amount = 800_000_000; // 0.8 SOL

    let offer_ix = create_make_offer_ix(
        &buyer.pubkey(),   // buyer: signer, pays SOL into escrow
        &asset.pubkey(),   // asset: read-only, for PDA derivation
        &listing,          // listing: read-only, must exist
        &offer,            // offer: PDA to be created
        offer_amount,      // amount: 0.8 SOL (instruction arg)
    );

    // Only buyer signs — offer PDA is created by Anchor (init)
    let res = send(&mut svm, &[offer_ix], &buyer, &[&buyer]);
    assert!(res.is_ok(), "make_offer failed: {:?}", res.err());
}