use borsh::{BorshDeserialize, BorshSerialize};

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
