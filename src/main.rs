use anyhow::Result;
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(name = "cranker-expense")]
#[command(about = "Analyze Solana cranker account expenses by program", long_about = None)]
struct Args {
    /// The cranker account address to analyze
    #[arg(short, long)]
    address: String,

    /// Number of transactions to fetch (max 1000)
    #[arg(short, long, default_value = "100")]
    limit: usize,

    /// RPC endpoint URL
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: String,

    /// Output CSV file path
    #[arg(short, long, default_value = "cranker_expenses.csv")]
    output: String,
}

#[derive(Debug, Clone)]
struct ProgramExpense {
    program_id: String,
    transaction_count: usize,
    total_fees_lamports: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Analyzing cranker account: {}", args.address);
    println!("📡 Using RPC: {}\n", args.rpc_url);

    let client = RpcClient::new(args.rpc_url);
    let pubkey = Pubkey::from_str(&args.address)?;

    // Fetch signatures
    println!("⏳ Fetching last {} transactions...", args.limit);
    let signatures = client.get_signatures_for_address(&pubkey)?;
    let limit = std::cmp::min(signatures.len(), args.limit);

    let mut program_expenses: HashMap<String, ProgramExpense> = HashMap::new();
    let mut total_fees = 0u64;

    println!("📊 Processing transactions...\n");

    for (i, sig_info) in signatures.iter().take(limit).enumerate() {
        if let Ok(signature) = Signature::from_str(&sig_info.signature) {
            if let Ok(tx) = client.get_transaction(&signature, UiTransactionEncoding::Json) {
                if let Some(meta) = tx.transaction.meta {
                    let fee = meta.fee;
                    total_fees += fee;

                    // Extract program IDs from transaction
                    if let Some(transaction) = tx.transaction.transaction.decode() {
                        let message = transaction.message;
                        let account_keys = message.static_account_keys();

                        for instruction in message.instructions() {
                            let program_id =
                                account_keys[instruction.program_id_index as usize].to_string();

                            program_expenses
                                .entry(program_id.clone())
                                .and_modify(|e| {
                                    e.transaction_count += 1;
                                    e.total_fees_lamports += fee;
                                })
                                .or_insert(ProgramExpense {
                                    program_id,
                                    transaction_count: 1,
                                    total_fees_lamports: fee,
                                });
                        }
                    }
                }
            }
        }

        if (i + 1) % 10 == 0 {
            println!("  Processed {}/{} transactions", i + 1, limit);
        }
    }

    // Sort by total fees descending
    let mut expenses: Vec<_> = program_expenses.into_iter().map(|(_, v)| v).collect();
    expenses.sort_by(|a, b| b.total_fees_lamports.cmp(&a.total_fees_lamports));

    // Display results
    println!("\n{}", "=".repeat(80));
    println!("📈 EXPENSE BREAKDOWN BY PROGRAM");
    println!("{}\n", "=".repeat(80));

    println!(
        "{:<45} {:>12} {:>15}",
        "Program ID", "Tx Count", "Total Fees (SOL)"
    );
    println!("{:-<80}", "");

    for expense in expenses.clone() {
        let sol_amount = expense.total_fees_lamports as f64 / 1e9;
        println!(
            "{:<45} {:>12} {:>15.9}",
            &expense.program_id[..std::cmp::min(44, expense.program_id.len())],
            expense.transaction_count,
            sol_amount
        );
    }

    println!("{:-<80}", "");
    println!(
        "{:<45} {:>12} {:>15.9}",
        "TOTAL",
        limit,
        total_fees as f64 / 1e9
    );
    println!("{}\n", "=".repeat(80));

    // Export to CSV
    println!("💾 Exporting to CSV: {}", args.output);
    export_to_csv(&expenses.clone(), &args.output)?;
    println!("✅ Export complete!\n");

    Ok(())
}

fn export_to_csv(expenses: &[ProgramExpense], filepath: &str) -> Result<()> {
    let mut file = File::create(filepath)?;

    // Write header
    writeln!(
        file,
        "program_id,transaction_count,total_fees_lamports,total_fees_sol"
    )?;

    // Write data
    for expense in expenses {
        writeln!(
            file,
            "{},{},{},{:.9}",
            expense.program_id,
            expense.transaction_count,
            expense.total_fees_lamports,
            expense.total_fees_lamports as f64 / 1e9
        )?;
    }

    Ok(())
}
