use std::str::FromStr;

use borsh::BorshDeserialize;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use glass::movie::{Staking, StakingAccountState};
use mpl_token_metadata::types::TokenStandard;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::{lamports_to_sol, LAMPORTS_PER_SOL},
    pubkey::Pubkey,
    system_program,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair, signer::Signer, system_transaction, transaction::Transaction,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize account
    InitializeStakeAccount { program_id: String, token: String },

    /// Stake
    Stake { program_id: String, token: String },

    /// Redeem
    Redeem { program_id: String, token: String },

    /// Unstake
    Unstake { program_id: String, token: String },

    /// Get staking
    GetStaking { pubkey: String },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    let url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(url.to_string());

    match &cli.command {
        Commands::InitializeStakeAccount { program_id, token } => {
            let signer = initialize_keypair();
            let program_id = Pubkey::from_str(&program_id).expect("parse program_id to Pubkey");
            let token = Pubkey::from_str(&token).expect("parse token to Pubkey");
            let staking = Staking::new(0, token);

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
        Commands::Stake { program_id, token } => {}
        Commands::Redeem { program_id, token } => {}
        Commands::Unstake { program_id, token } => {}
        Commands::GetStaking { pubkey } => {
            let pubkey = Pubkey::from_str(pubkey).expect("parse to Pubkey");
            let account_data = rpc_client
                .get_account_data(&pubkey)
                .await
                .expect("get account");

            let staking = StakingAccountState::deserialize(&mut account_data.as_ref())
                .expect("deserialize staking");
            eprintln!("Staking is_initialized: {:?}", staking.is_initialized);
            eprintln!("Staking token: {:?}", staking.token);
            eprintln!("Staking insert_date: {:?}", staking.insert_date);
        }
    }
}

fn initialize_keypair() -> Keypair {
    match std::env::var("PRIVATE_KEY") {
        Ok(private_key) => {
            println!("Found a keypair");
            Keypair::from_base58_string(&private_key)
        }
        Err(_) => {
            println!("Generating new keypair...");
            let signer = Keypair::new();
            std::fs::write(".env", format!("PRIVATE_KEY={}", signer.to_base58_string()))
                .expect("write secret key");

            signer
        }
    }
}

async fn airdrop_sol_if_needed(signer: &Pubkey, connection: &RpcClient) {
    let balance = connection.get_balance(&signer).await.expect("get balance");
    println!("Current balance is {} SOL", balance / LAMPORTS_PER_SOL);

    if balance / LAMPORTS_PER_SOL < 1 {
        println!("Airdropping 1 SOL");

        let airdrop_sig = connection
            .request_airdrop(&signer, LAMPORTS_PER_SOL)
            .await
            .expect("request airdrop");

        loop {
            let confirmed = connection
                .confirm_transaction(&airdrop_sig)
                .await
                .expect("confirm transaction");
            if confirmed {
                break;
            }
        }

        let balance = connection.get_balance(&signer).await.expect("get balance");
        println!("New balance is {} SOL", balance / LAMPORTS_PER_SOL);
    }
}

async fn ping_program(
    connection: &RpcClient,
    payer: &Keypair,
    program_id: Pubkey,
    program_data_pkey: &Pubkey,
) {
    let accounts = vec![AccountMeta::new(*program_data_pkey, false)];
    let instruction = Instruction::new_with_bytes(program_id, &[], accounts);
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = connection
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");

    let transaction = Transaction::new(&[payer], message, recent_blockhash);
    let transaction_sig = connection
        .send_and_confirm_transaction(&transaction)
        .await
        .expect("send and confirm transaction");

    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        transaction_sig
    );
}

async fn transfer_sol(connection: &RpcClient, from: &Keypair, to: &Pubkey) {
    let recent_blockhash = connection
        .get_latest_blockhash()
        .await
        .expect("get latest block hash");
    let tx = system_transaction::transfer(&from, to, LAMPORTS_PER_SOL / 10, recent_blockhash);
    let sig = connection
        .send_and_confirm_transaction(&tx)
        .await
        .expect("send and confirm transaction");

    println!(
        "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
        sig
    );
}
