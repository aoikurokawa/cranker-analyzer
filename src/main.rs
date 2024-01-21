use std::str::FromStr;

use borsh::BorshDeserialize;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use glass::{
    asset::Asset,
    movie::{Movie, MovieAccountState},
};
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
    /// print balance
    Balance { pkey: String },

    /// Counter Program
    Counter { program_id: Pubkey, pkey: Pubkey },

    /// Transfer SOL one account to another
    Transfer { to: Pubkey },

    /// Add movie review
    AddMovieReview {
        program_id: String,
        title: String,
        rating: u8,
        description: String,
    },

    /// Get movie review
    GetMovieReview { pubkey: String },

    /// Get movie review
    UpdateMovieReview {
        program_id: String,
        title: String,
        rating: u8,
        description: String,
    },

    /// Mint a Token
    TokenMinter { name: String, uri: String },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    let url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(url.to_string());

    match &cli.command {
        Commands::Balance { pkey } => {
            let pkey = Pubkey::from_str(&pkey).expect("parse to Pubkey");
            let balance = rpc_client.get_balance(&pkey).await.expect("get balance");

            println!("Balance: {}SOL", lamports_to_sol(balance));
        }
        Commands::Counter { program_id, pkey } => {
            let signer = initialize_keypair();

            println!("Public key: {}", signer.pubkey().to_string());

            airdrop_sol_if_needed(&signer.pubkey(), &rpc_client).await;

            ping_program(&rpc_client, &signer, *program_id, pkey).await;
            println!("Finished successfully");
        }
        Commands::Transfer { to } => {
            let signer = initialize_keypair();

            println!("Public key: {}", signer.pubkey().to_string());

            // airdrop_sol_if_needed(&signer.pubkey(), &rpc_client);

            transfer_sol(&rpc_client, &signer, to).await;

            println!("send 0.1 SOL");
        }
        Commands::AddMovieReview {
            program_id,
            title,
            rating,
            description,
        } => {
            let signer = initialize_keypair();
            let program_id = Pubkey::from_str(&program_id).expect("parse to Pubkey");

            let movie = Movie::new(0, title.to_string(), *rating, description.to_string());

            let (pda, _bump) = Pubkey::find_program_address(
                &[&signer.pubkey().to_bytes(), title.as_bytes()],
                &program_id,
            );
            let accounts = vec![
                AccountMeta::new(signer.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new(system_program::id(), false),
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
        Commands::GetMovieReview { pubkey } => {
            let pubkey = Pubkey::from_str(pubkey).expect("parse to Pubkey");
            let account_data = rpc_client
                .get_account_data(&pubkey)
                .await
                .expect("get account");

            let movie = MovieAccountState::deserialize(&mut account_data.as_ref())
                .expect("deserialize movie");
            eprintln!("Movie is_initialized: {:?}", movie.is_initialized);
            eprintln!("Movie rating: {:?}", movie.rating);
            eprintln!("Movie title: {:?}", movie.title);
            eprintln!("Movie description: {:?}", movie.description);
        }
        Commands::UpdateMovieReview {
            program_id,
            title,
            rating,
            description,
        } => {
            let signer = initialize_keypair();
            let program_id = Pubkey::from_str(&program_id).expect("parse to Pubkey");

            let movie = Movie::new(1, title.to_string(), *rating, description.to_string());

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
        Commands::TokenMinter { name, uri } => {
            let mut asset = Asset::default();
            let token_standard = TokenStandard::NonFungible;
            let payer = initialize_keypair();
            let spl_token_program = spl_token::id();

            asset
                .create(
                    &rpc_client,
                    name.to_string(),
                    uri.to_string(),
                    token_standard,
                    &payer,
                    &payer,
                    spl_token_program,
                )
                .await;

            let token_owner = Keypair::new().pubkey();

            asset
                .mint(
                    &rpc_client,
                    &token_owner,
                    1,
                    &payer,
                    &payer,
                    spl_token_program,
                )
                .await;
        }
    }
}

fn initialize_keypair() -> Keypair {
    match std::env::var("PRIVATE_KEY") {
        Ok(private_key) => {
            print!("Found a keypair");
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
