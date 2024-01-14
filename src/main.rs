use std::{fs, str::FromStr};

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use solana_program::{
    native_token::{lamports_to_sol, LAMPORTS_PER_SOL},
    pubkey::Pubkey,
};
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{signature::Keypair, signer::Signer};

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
    Counter { program_id: String, pkey: Pubkey },
}

fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    let url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(url);

    match &cli.command {
        Commands::Balance { pkey } => {
            let pkey = Pubkey::from_str(&pkey).expect("parse to Pubkey");
            let balance = rpc_client.get_balance(&pkey).expect("get balance");

            println!("Balance: {}SOL", lamports_to_sol(balance));
        }
        Commands::Counter { program_id, pkey } => {
            let signer = initialize_keypair();

            println!("Public key: {}", signer.pubkey().to_string());

            airdrop_sol_if_needed(&signer.pubkey(), &rpc_client);
            println!("Finished successfully");
        }
    }
}

fn initialize_keypair() -> Keypair {
    match std::env::var("PRIVATE_KEY") {
        Ok(private_key) => {
            println!("Generating new keypair...");
            Keypair::from_base58_string(&private_key)
        }
        Err(_) => {
            let signer = Keypair::new();
            std::fs::write(".env", format!("PRIVATE_KEY={}", signer.to_base58_string()))
                .expect("write secret key");

            signer
        }
    }
}

fn airdrop_sol_if_needed(signer: &Pubkey, connection: &RpcClient) {
    let balance = connection.get_balance(&signer).expect("get balance");
    println!("Current balance is {} SOL", balance / LAMPORTS_PER_SOL);

    if balance / LAMPORTS_PER_SOL < 1 {
        println!("Airdropping 1 SOL");

        let airdrop_sig = connection
            .request_airdrop(&signer, LAMPORTS_PER_SOL)
            .expect("request airdrop");
        let block_hash = connection
            .get_latest_blockhash()
            .expect("get latest blockhash");

        loop {
            let confirmed = connection
                .confirm_transaction(&airdrop_sig)
                .expect("confirm transaction");
            if confirmed {
                break;
            }
        }

        let balance = connection.get_balance(&signer).expect("get balance");
        println!("New balance is {} SOL", balance / LAMPORTS_PER_SOL);
    }
}
