use anyhow::Result;
use clap::Parser;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction_status::{EncodedTransaction, UiParsedInstruction, UiTransactionEncoding};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "cranker-expense")]
#[command(about = "Analyze Solana cranker account expenses by program", long_about = None)]
struct Args {
    /// The cranker account address to analyze
    #[arg(short, long)]
    address: String,

    /// Number of transactions to fetch (supports pagination for >1000)
    #[arg(short, long, default_value = "100")]
    limit: usize,

    /// RPC endpoint URL
    #[arg(short, long)]
    rpc_url: String,

    /// Output CSV file path
    #[arg(short, long, default_value = "cranker_expenses.csv")]
    output: String,

    /// Concurrent requests
    #[arg(short = 'c', long, default_value = "50")]
    concurrency: usize,
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
    let start = Instant::now();

    println!("🔍 Analyzing cranker account: {}", args.address);
    println!("📡 Using RPC: {}", args.rpc_url);
    println!("⚡ Concurrency: {}\n", args.concurrency);

    let client =
        RpcClient::new_with_commitment(args.rpc_url.clone(), CommitmentConfig::confirmed());
    let pubkey = Pubkey::from_str(&args.address)?;

    // Fetch signatures with pagination
    println!("⏳ Fetching signatures...");
    let mut all_signatures = Vec::new();
    let mut before_signature = None;

    while all_signatures.len() < args.limit {
        let batch = if let Some(before) = before_signature {
            client
                .get_signatures_for_address_with_config(
                    &pubkey,
                    solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                        before: Some(before),
                        limit: Some(1000),
                        ..Default::default()
                    },
                )
                .await?
        } else {
            client.get_signatures_for_address(&pubkey).await?
        };

        if batch.is_empty() {
            break; // No more signatures
        }

        before_signature = batch
            .last()
            .map(|s| Signature::from_str(&s.signature).unwrap());

        all_signatures.extend(batch);

        if all_signatures.len() >= args.limit {
            all_signatures.truncate(args.limit);
            break;
        }

        if all_signatures.len() > 1000 {
            println!("  Fetched {} signatures so far...", all_signatures.len());
        }
    }

    let limit = all_signatures.len();
    println!("✓ Found {} signatures\n", limit);

    // Fetch transactions concurrently in batches
    println!(
        "📊 Fetching {} transactions with {} concurrent requests...",
        limit, args.concurrency
    );

    let mut program_expenses: HashMap<String, ProgramExpense> = HashMap::new();
    let mut total_fees = 0u64;
    let mut processed = 0;

    // Process in chunks to avoid overwhelming the RPC
    let chunk_size = args.concurrency;
    let chunks: Vec<_> = all_signatures
        .iter()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect();

    for (_chunk_idx, chunk) in chunks.iter().enumerate() {
        let mut tasks = vec![];

        for sig_info in chunk {
            let client =
                RpcClient::new_with_commitment(args.rpc_url.clone(), CommitmentConfig::confirmed());
            let signature_str = sig_info.signature.clone();

            let task = tokio::spawn(async move {
                if let Ok(signature) = Signature::from_str(&signature_str) {
                    match client
                        .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
                        .await
                    {
                        Ok(tx) => {
                            let mut program_ids = Vec::new();
                            let mut fee = 0;

                            if let Some(meta) = &tx.transaction.meta {
                                fee = meta.fee;
                            }

                            if let EncodedTransaction::Json(ui_tx) = tx.transaction.transaction {
                                // Handle parsed message format
                                if let solana_transaction_status::UiMessage::Parsed(parsed_msg) =
                                    ui_tx.message
                                {
                                    for instruction in &parsed_msg.instructions {
                                        match instruction {
                                            solana_transaction_status::UiInstruction::Parsed(
                                                parsed_ix,
                                            ) => {
                                                if let UiParsedInstruction::Parsed(parsed_ix) =
                                                    parsed_ix
                                                {
                                                    program_ids.push(parsed_ix.program_id.clone());
                                                }
                                            }
                                            solana_transaction_status::UiInstruction::Compiled(
                                                compiled_ix,
                                            ) => {
                                                let idx = compiled_ix.program_id_index as usize;
                                                if idx < parsed_msg.account_keys.len() {
                                                    program_ids.push(
                                                        parsed_msg.account_keys[idx].pubkey.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                // Handle raw message format
                                else if let solana_transaction_status::UiMessage::Raw(raw_msg) =
                                    ui_tx.message
                                {
                                    for instruction in &raw_msg.instructions {
                                        let idx = instruction.program_id_index as usize;
                                        if idx < raw_msg.account_keys.len() {
                                            program_ids.push(raw_msg.account_keys[idx].clone());
                                        }
                                    }
                                }

                                if !program_ids.is_empty() {
                                    return Some((fee, program_ids));
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
                None
            });

            tasks.push(task);
        }

        // Wait for all tasks in this chunk to complete
        let results = futures::future::join_all(tasks).await;

        for result in results {
            if let Ok(Some((fee, program_ids))) = result {
                total_fees += fee;
                processed += 1;

                for program_id in program_ids {
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

        println!(
            "  Progress: {}/{} transactions ({:.1}%)",
            processed,
            limit,
            (processed as f64 / limit as f64) * 100.0
        );
    }

    let duration = start.elapsed();
    println!("\n✓ Completed in {:.2}s\n", duration.as_secs_f64());

    // Sort by total fees descending
    let mut expenses: Vec<_> = program_expenses.into_iter().map(|(_, v)| v).collect();
    expenses.sort_by(|a, b| b.total_fees_lamports.cmp(&a.total_fees_lamports));

    // Display results
    println!("{}", "=".repeat(80));
    println!("📈 EXPENSE BREAKDOWN BY PROGRAM");
    println!("{}\n", "=".repeat(80));

    println!(
        "{:<45} {:>12} {:>15}",
        "Program ID", "Tx Count", "Total Fees (SOL)"
    );
    println!("{:-<80}", "");

    for expense in &expenses {
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
    export_to_csv(&expenses, &args.output)?;
    println!("✅ Export complete!\n");

    Ok(())
}

fn export_to_csv(expenses: &[ProgramExpense], filepath: &str) -> Result<()> {
    let mut file = File::create(filepath)?;

    writeln!(
        file,
        "program_id,transaction_count,total_fees_lamports,total_fees_sol"
    )?;

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
