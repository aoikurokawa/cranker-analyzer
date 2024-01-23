use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey, clock::UnixTimestamp};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StakeInfo {
    varint: u8,
}

impl StakeInfo {
    pub fn new(varint: u8) -> Self {
        Self { varint }
    }
}

#[derive(BorshDeserialize)]
pub struct StakeInfoAccountState {
    pub is_initialized: bool,
    pub token_account: Pubkey,
    pub stake_start_time: UnixTimestamp,
    pub last_stake_redeem: UnixTimestamp,
    pub user_pubkey: Pubkey,
    pub stake_state: StakeState,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum StakeState {
    Staked,
    Unstaked,
}
