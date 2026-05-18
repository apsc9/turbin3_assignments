#[cfg(test)]
mod tests {
    use {
        anchor_escrow_q2_2026::program, anchor_lang::{
            AccountDeserialize, InstructionData, Key, ToAccountMetas, prelude::msg, pubkey, solana_program::{instruction::Instruction, program_pack::Pack}, system_program::ID as SYSTEM_PROGRAM_ID
        }, anchor_spl::{
            associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID}, mint, token::spl_token
        }, litesvm::LiteSVM, litesvm_token::{
            CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token::ID as TOKEN_PROGRAM_ID
        }, solana_keypair::Keypair, solana_message::{Address, Message}, solana_pubkey::Pubkey, solana_signer::Signer, solana_transaction::Transaction
    };

    // setup function to initialize liteSVM and create a payer keypair
    fn setup() -> (LiteSVM, Keypair) {
        let program_id = anchor_escrow_q2_2026::id();
        let payer = Keypair::new();
        let mut svm = LiteSVM::new();
        let bytes = include_bytes!("../../../target/deploy/anchor_escrow_q2_2026.so");
        svm.add_program(program_id, bytes).unwrap();
        svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

        (svm, payer)
    }

    #[test]
    fn test_make_and_refund() {

        let (mut program, payer) = setup();
        let maker = payer.pubkey();

        // create 2 mints(mintA and mintB) with 6 decimal places and maker as authority
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint A : {}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint B : {}\n", mint_b);

        // create maker's ata for mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // derive the PDA for the escrow account using the maker's public key 
        // and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()], 
            &anchor_escrow_q2_2026::id()
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        // derive pda for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: anchor_escrow_q2_2026::id(),
            accounts: anchor_escrow_q2_2026::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: anchor_escrow_q2_2026::instruction::Make {
                deposit: 10_000_000,
                seed: 123u64,
                receive: 10_000_000,
            }
            .data()
        };

        // create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction anc capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction successful");
        msg!("CUs consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10_000_000);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data = anchor_escrow_q2_2026::state::Escrow::try_deserialize(
            &mut escrow_account.data.as_ref(),
        )
        .unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10_000_000);


        // create the refund instruction to refund tokens back to their maker
        let refund_ix = Instruction {
            program_id: anchor_escrow_q2_2026::id(),
            accounts: anchor_escrow_q2_2026::accounts::Refund {
                maker: maker,
                mint_a: mint_a,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID
            }
            .to_account_metas(None),
            data: anchor_escrow_q2_2026::instruction::Refund {}.data(),
        };

        // create and send the transaction containing the "Refund" instruction
        let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        
        // send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // log transaction details
        msg!("\n\n Refund Transaction Successful");
        msg!("CUs consumed: {}", tx.compute_units_consumed);
        msg!("Tx signature: {}", tx.signature);

        assert!(program.get_account(&escrow).is_none());
        assert!(program.get_account(&vault).is_none());
    
    }

    #[test]
    fn test_make_and_take() {
        let (mut program, payer) = setup();
        

        // Get maker's public key from payer keypair
        let maker = payer.pubkey();
        let taker = Keypair::new();
        
        program.airdrop(&taker.pubkey(), 1_000_000_000).unwrap();

        // create mints
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint A: {}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        msg!("Mint B: {}\n", mint_b);

        // create maker ATA for mint A 
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // create maker ATA for mint B
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&maker)
            .send()
            .unwrap();
        msg!("Maker ATA B: {}\n", maker_ata_b);


        // create taker ATA for mint A
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA A: {}\n", taker_ata_a);

        // create taker ATA for mint B
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);

        // derive escrow PDA using maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
            &anchor_escrow_q2_2026::id(),
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        // derive PDA for vault associated token account using escrow PDA and mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // mint 1000 tokens (with 6 decimal places) of Mint A to maker's ATA
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
            .send()
            .unwrap();

        // mint 1000 tokens (with 6 decimal places) of Mint B to taker's ATA
        MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000_000_000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: anchor_escrow_q2_2026::id(),
            accounts: anchor_escrow_q2_2026::accounts::Make {
                maker,
                mint_a,
                mint_b,
                maker_ata_a,
                escrow,
                vault,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: anchor_escrow_q2_2026::instruction::Make {
                deposit: 10_000_000,
                seed: 123u64,
                receive: 10_000_000,
            }
            .data(),
        };

        // create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();


        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        
        // send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log Transaction details

        msg!("\nMake transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx signature: {}", tx.signature);

        // verify vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10_000_000);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data = anchor_escrow_q2_2026::state::Escrow::try_deserialize(
            &mut escrow_account.data.as_ref(),
        )
        .unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10_000_000);


        // Create  the "Take" instruction to execute the trade and close the escrow
        let take_ix = Instruction {
            program_id: anchor_escrow_q2_2026::id(),
            accounts: anchor_escrow_q2_2026::accounts::Take {
                taker: taker.pubkey(),
                maker,
                mint_a,
                mint_b,
                taker_ata_a,
                taker_ata_b,
                maker_ata_b,
                escrow,
                vault,
                token_program: TOKEN_PROGRAM_ID,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: anchor_escrow_q2_2026::instruction::Take {}.data(),
        };

        // create and send the transaction containing the "take" instruction
        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nTake Transaction successful ");
        msg!("CUs consumed: {}", tx.compute_units_consumed);
        msg!("Tx signature: {}", tx.signature);

        // escrow and vault closed
        assert!(program.get_account(&escrow).is_none());
        assert!(program.get_account(&vault).is_none());
    }

}

