use std::str::FromStr;

use borsh::BorshDeserialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    system_program,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::{
    initialize_keypair,
    staking::stake_info::{StakeInfo, StakeInfoAccountState},
};

pub mod stake_info;

pub async fn initialize_stake_account(rpc_client: &RpcClient, program_id: &str, token: &str) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(program_id).expect("parse program_id to Pubkey");
    let token = Pubkey::from_str(token).expect("parse token to Pubkey");
    let staking = StakeInfo::new(0);

    let (pda, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), token.to_bytes().as_ref()],
        &program_id,
    );
    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(token, false),
        AccountMeta::new(pda, false),
        AccountMeta::new(system_program::id(), false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &staking, accounts);
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

pub async fn stake(rpc_client: &RpcClient, program_id: &str, token: &str) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(&program_id).expect("parse program_id to Pubkey");
    let token = Pubkey::from_str(&token).expect("parse token to Pubkey");
    let staking = StakeInfo::new(1);

    let (pda, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), token.to_bytes().as_ref()],
        &program_id,
    );
    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(token, false),
        AccountMeta::new(pda, false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &staking, accounts);
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

pub async fn redeem(rpc_client: &RpcClient, program_id: &str, token: &str) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(&program_id).expect("parse program_id to Pubkey");
    let token = Pubkey::from_str(&token).expect("parse token to Pubkey");
    let staking = StakeInfo::new(2);

    let (pda, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), token.to_bytes().as_ref()],
        &program_id,
    );
    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(token, false),
        AccountMeta::new(pda, false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &staking, accounts);
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

pub async fn unstake(rpc_client: &RpcClient, program_id: &str, token: &str) {
    let signer = initialize_keypair();
    let program_id = Pubkey::from_str(&program_id).expect("parse program_id to Pubkey");
    let token = Pubkey::from_str(&token).expect("parse token to Pubkey");
    let staking = StakeInfo::new(3);

    let (pda, _bump) = Pubkey::find_program_address(
        &[&signer.pubkey().to_bytes(), token.to_bytes().as_ref()],
        &program_id,
    );
    let accounts = vec![
        AccountMeta::new(signer.pubkey(), true),
        AccountMeta::new(token, false),
        AccountMeta::new(pda, false),
    ];

    let instruction = Instruction::new_with_borsh(program_id, &staking, accounts);
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

pub async fn get_staking_info(rpc_client: &RpcClient, pubkey: &str) {
    let pubkey = Pubkey::from_str(pubkey).expect("parse to Pubkey");
    let account_data = rpc_client
        .get_account_data(&pubkey)
        .await
        .expect("get account");

    let staking = StakeInfoAccountState::deserialize(&mut account_data.as_ref())
        .expect("deserialize staking");

    eprintln!("Staking is_initialized: {:?}", staking.is_initialized);
    eprintln!("Staking token: {:?}", staking.token_account);
    eprintln!("Staking insert_date: {:?}", staking.stake_start_time);
    eprintln!("Staking stake_redeem: {:?}", staking.last_stake_redeem);
}
