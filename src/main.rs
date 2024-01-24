use clap::{Parser, Subcommand};
use dotenv::dotenv;
use glass::{movie, staking};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
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

    /// Add movie review
    AddMovieReview {
        program_id: String,
        title: String,
        rating: u8,
        description: String,
    },

    /// Get movie review
    GetMovieReview { pubkey: String },

    /// Update movie review
    UpdateMovieReview {
        program_id: String,
        title: String,
        rating: u8,
        description: String,
    },

    /// Initialize Token Mint
    InitializeTokenMint { program_id: String },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    let url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(url.to_string());

    match &cli.command {
        // Staking Program
        Commands::InitializeStakeAccount { program_id, token } => {
            staking::initialize_stake_account(&rpc_client, &program_id, &token).await;
        }
        Commands::Stake { program_id, token } => {
            staking::stake(&rpc_client, &program_id, &token).await;
        }
        Commands::Redeem { program_id, token } => {
            staking::redeem(&rpc_client, &program_id, &token).await;
        }
        Commands::Unstake { program_id, token } => {
            staking::unstake(&rpc_client, &program_id, &token).await;
        }
        Commands::GetStaking { pubkey } => {
            staking::get_staking_info(&rpc_client, &pubkey).await;
        }

        // Movie Review
        Commands::AddMovieReview {
            program_id,
            title,
            rating,
            description,
        } => {
            movie::add_movie_review(&rpc_client, &program_id, &title, *rating, &description).await;
        }
        Commands::GetMovieReview { pubkey } => {
            movie::get_movie_review(&rpc_client, &pubkey).await;
        }
        Commands::UpdateMovieReview {
            program_id,
            title,
            rating,
            description,
        } => {
            movie::update_movie_review(&rpc_client, &program_id, &title, *rating, &description)
                .await;
        }
        Commands::InitializeTokenMint { program_id } => {
            movie::initialize_token_mint(&rpc_client, &program_id).await;
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
