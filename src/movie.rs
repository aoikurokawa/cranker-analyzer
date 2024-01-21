use borsh::{BorshDeserialize, BorshSerialize};
use serde::Deserialize;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Movie {
    varint: u8,
    title: String,
    rating: u8,
    description: String,
}

impl Movie {
    pub fn new(title: String, rating: u8, description: String) -> Self {
        Self {
            varint: 0,
            title,
            rating,
            description,
        }
    }
}

#[derive(BorshDeserialize)]
pub struct MovieAccountState {
    pub is_initialized: bool,
    pub rating: u8,
    pub title: String,
    pub description: String,
}
