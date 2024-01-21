use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Staking {
    varint: u8,
    token: Pubkey,
}

impl Staking {
    pub fn new(varint: u8, token: Pubkey) -> Self {
        Self { varint, token }
    }
}

#[derive(BorshDeserialize)]
pub struct StakingAccountState {
    pub is_initialized: bool,
    pub token: Pubkey,
    pub insert_date: i64,
}
