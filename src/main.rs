use std::str::FromStr;

use clap::{Parser, Subcommand};
use solana_program::{native_token::lamports_to_sol, pubkey::Pubkey};
use solana_rpc_client::rpc_client::RpcClient;

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
}

fn main() {
    let cli = Cli::parse();
    let url = "https://api.devnet.solana.com";
    let rpc_client = RpcClient::new(url);

    match &cli.command {
        Commands::Balance { pkey } => {
            let pkey = Pubkey::from_str(&pkey).expect("parse to Pubkey");
            let balance = rpc_client.get_balance(&pkey).expect("get balance");

            println!("Balance: {}SOL", lamports_to_sol(balance));
        }
    }
}
