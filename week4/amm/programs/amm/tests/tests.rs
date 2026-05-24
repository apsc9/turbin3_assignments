
use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Address, Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

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

// setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // create 2 mints(A and B) with 6 decimal places and maker as the authority
    // this is done using litesvm-token's createMint utility which creates the mint in the 
    // LiteSVM env
    let mint_x: Address = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y: Address = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let seed_bytes = 123u64.to_le_bytes();
    let config: Address = Pubkey::find_program_address(&[b"config", seed_bytes.as_ref()],  &amm::id()).0 ;
    let mint_lp: Address = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm::id()).0 ;

    // derive the PDA for the vault associated token account using the config PDA and mint A
    let vault_x: Address = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y: Address = associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    )
}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());

    // fetch state accounts(config, vault and so on) and 
    // make sure that the data we expect is what is present
}

#[test]
fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
fn test_withdraw() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let withdraw_ix = create_withdraw_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y);

    let res = send(
        &mut svm, 
        &[init_ix, deposit_ix, withdraw_ix], 
        &payer, 
        &[&payer]
    );
    assert!(res.is_ok());
}

#[test]
fn test_swap() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y
    );

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y
    );

    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, swap_ix],
        &payer,
        &[&payer]
    );

    assert!(res.is_ok());
}
