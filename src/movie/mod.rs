use std::str::FromStr;

use borsh::BorshDeserialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::client_error::ErrorKind;
use solana_sdk::{
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    initialize_keypair,
    movie::movie_account::{Movie, MovieAccountState},
};

mod movie_account;

pub async fn add_movie_review(
    rpc_client: &RpcClient,
    program_id: &str,
    title: &str,
    rating: u8,
    description: &str,
) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(program_id).expect("parse to Pubkey");

    let movie = Movie::new(0, title.to_string(), rating, description.to_string());

    let (pda_account, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), title.as_bytes()],
        &program_id,
    );

    let (pda_counter, _bump) =
        Pubkey::find_program_address(&[pda_account.as_ref(), "comment".as_ref()], &program_id);

    let (token_mint, _token_mint_bump) =
        Pubkey::find_program_address(&[b"token_mint"], &program_id);

    let (mint_auth, _mint_auth_bump) = Pubkey::find_program_address(&[b"token_auth"], &program_id);

    let user_ata = get_associated_token_address(&signer.pubkey(), &token_mint);
    eprintln!("User associated token account: {:?}", user_ata);

    if !exist_user_associated_token_account(rpc_client, &user_ata).await {
        eprintln!("Does not exist user associated token account, so we will create it");
        initialize_user_associated_token_account(rpc_client, &program_id).await;
    }

    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(pda_account, false),
        AccountMeta::new(pda_counter, false),
        AccountMeta::new(token_mint, false),
        AccountMeta::new(mint_auth, false),
        AccountMeta::new(user_ata, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(spl_token::id(), false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &movie, accounts);
    let message = Message::new(&[instruction], Some(&signer.pubkey()));
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");
    let transaction = Transaction::new(&[&signer], message, recent_blockhash);
    let transaction_sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("send and confirm transaction");
    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        transaction_sig
    );
}

pub async fn get_movie_review(rpc_client: &RpcClient, pubkey: &str) {
    let pubkey = Pubkey::from_str(pubkey).expect("parse to Pubkey");
    let account_data = rpc_client
        .get_account_data(&pubkey)
        .await
        .expect("get account");

    let movie =
        MovieAccountState::deserialize(&mut account_data.as_ref()).expect("deserialize movie");
    eprintln!("Movie is_initialized: {:?}", movie.is_initialized);
    eprintln!("Movie rating: {:?}", movie.rating);
    eprintln!("Movie title: {:?}", movie.title);
    eprintln!("Movie description: {:?}", movie.description);
    eprintln!("Movie description: {:?}", movie.description);
}

pub async fn update_movie_review(
    rpc_client: &RpcClient,
    program_id: &str,
    title: &str,
    rating: u8,
    description: &str,
) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(&program_id).expect("parse to Pubkey");

    let movie = Movie::new(1, title.to_string(), rating, description.to_string());

    let (pda, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), title.as_bytes()],
        &program_id,
    );
    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(pda, false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &movie, accounts);
    let message = Message::new(&[instruction], Some(&signer.pubkey()));
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");

    let transaction = Transaction::new(&[&signer], message, recent_blockhash);
    let transaction_sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("send and confirm transaction");
    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        transaction_sig
    );
}

pub async fn initialize_token_mint(rpc_client: &RpcClient, program_id: &str) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(program_id).expect("parse to Pubkey");

    let varint = Movie::new(3, "".to_string(), 0, "".to_string());
    let (token_mint, _token_mint_bump) =
        Pubkey::find_program_address(&[b"token_mint"], &program_id);

    let (mint_auth, _mint_auth_bump) = Pubkey::find_program_address(&[b"token_auth"], &program_id);

    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(token_mint, false),
        AccountMeta::new(mint_auth, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(spl_token::id(), false),
        AccountMeta::new(sysvar::rent::id(), false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &varint, accounts);
    let message = Message::new(&[instruction], Some(&signer.pubkey()));
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");

    let transaction = Transaction::new(&[&signer], message, recent_blockhash);
    let transaction_sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("send and confirm transaction");
    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        transaction_sig
    );
}

async fn exist_user_associated_token_account(rpc_client: &RpcClient, pubkey: &Pubkey) -> bool {
    match rpc_client.get_account(pubkey).await {
        Ok(_) => {}
        Err(e) => match e.kind {
            ErrorKind::TransactionError(tx_err) => {
                if tx_err == TransactionError::AccountNotFound {
                    return true;
                }
            }
            _ => {}
        },
    }

    return false;
}

pub async fn initialize_user_associated_token_account(rpc_client: &RpcClient, program_id: &Pubkey) {
    let signer = initialize_keypair();

    let (token_mint, _token_mint_bump) =
        Pubkey::find_program_address(&[b"token_mint"], &program_id);

    let instruction = spl_associated_token_account::instruction::create_associated_token_account(
        &signer.pubkey(),
        &signer.pubkey(),
        &token_mint,
        &spl_token::id(),
    );
    let message = Message::new(&[instruction], Some(&signer.pubkey()));
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");

    let transaction = Transaction::new(&[&signer], message, recent_blockhash);
    let transaction_sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("send and confirm transaction");
    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        transaction_sig
    );
}
