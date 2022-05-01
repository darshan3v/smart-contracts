use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: String,
    pub reference_hash: Base64VecU8,
    pub decimals: u8,
}

pub trait FungibleTokenMetadataProvider {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    /// Return Metadata related to the Fungible Token
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.ft_metadata.get().unwrap().clone()
    }
}

impl FungibleTokenMetadata {
    pub fn assert_valid_metadata(&self) {
        require!(self.reference_hash.0.len() == 32, "Hash has to be 32 bytes");
    }
}
