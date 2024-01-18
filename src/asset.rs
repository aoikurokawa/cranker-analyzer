use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata, TokenRecord},
    instructions::{CreateV1Builder, MintV1Builder},
    types::{PrintSupply, TokenStandard},
};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

pub struct Asset {
    pub mint: Keypair,
    pub metadata: Pubkey,
    pub master_edition: Pubkey,
    pub token: Pubkey,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            mint: Keypair::new(),
            metadata: Pubkey::default(),
            master_edition: Pubkey::default(),
            token: Pubkey::default(),
        }
    }
}

impl Asset {
    pub async fn create(
        &mut self,
        rpc_client: &RpcClient,
        name: String,
        uri: String,
        token_standard: TokenStandard,
        update_authority: &Keypair,
        payer: &Keypair,
        spl_token_program: Pubkey,
    ) {
        self.metadata = Metadata::find_pda(&self.mint.pubkey()).0;
        self.master_edition = MasterEdition::find_pda(&self.mint.pubkey()).0;

        let create_ix = CreateV1Builder::new()
            .metadata(self.metadata)
            .master_edition(Some(self.master_edition))
            .mint(self.mint.pubkey(), true)
            .authority(update_authority.pubkey())
            .payer(payer.pubkey())
            .update_authority(update_authority.pubkey(), true)
            .is_mutable(true)
            .primary_sale_happened(false)
            .seller_fee_basis_points(500)
            .print_supply(PrintSupply::Zero)
            .name(name)
            .uri(uri)
            .token_standard(token_standard)
            .spl_token_program(Some(spl_token_program))
            .instruction();
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("get latest block hash");

        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&payer.pubkey()),
            &[payer, update_authority, &self.mint],
            recent_blockhash,
        );

        let transaction_sig = rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .expect("send and confirm transaction");
        println!(
            "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
            transaction_sig
        );
    }

    pub async fn mint(
        &mut self,
        rpc_client: &RpcClient,
        token_owner: &Pubkey,
        amount: u64,
        update_authority: &Keypair,
        payer: &Keypair,
        spl_token_program: Pubkey,
    ) {
        if self.token == Pubkey::default() {
            self.token = spl_associated_token_account::get_associated_token_address_with_program_id(
                token_owner,
                &self.mint.pubkey(),
                &spl_token_program,
            );
        }

        let token_record = TokenRecord::find_pda(&self.mint.pubkey(), &self.token).0;

        let mint_ix = MintV1Builder::new()
            .token(self.token)
            .token_owner(Some(*token_owner))
            .metadata(self.metadata)
            .master_edition(Some(self.master_edition))
            .token_record(Some(token_record))
            .mint(self.mint.pubkey())
            .authority(update_authority.pubkey())
            .payer(payer.pubkey())
            .amount(amount)
            .spl_token_program(spl_token_program)
            .instruction();

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("get latest block hash");

        let tx = Transaction::new_signed_with_payer(
            &[mint_ix],
            Some(&payer.pubkey()),
            &[payer, update_authority],
            recent_blockhash,
        );

        let transaction_sig = rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .expect("send and confirm transaction");
        println!(
            "Transaction https://explorer.solana.com/tx/{}?cluster=devnet",
            transaction_sig
        );
    }
}
