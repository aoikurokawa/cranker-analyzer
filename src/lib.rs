use solana_sdk::signature::Keypair;

pub mod asset;
pub mod movie;
pub mod staking;

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
