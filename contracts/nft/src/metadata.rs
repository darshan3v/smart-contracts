use crate::*;

/// TokenId = EventId.TokenId
pub type TokenId = String;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub base_uri: String, // Decentralized storage gateway
    pub reference: String,
    pub reference_hash: Base64VecU8,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: String,
    pub description: Option<String>, // free-form description
    pub media: String,
    pub media_hash: Base64VecU8,
    pub copies: Option<u64>, // max number of copies of this set of metadata that can be minted.
    pub issued_at: Option<u64>, // Unix epoch in milliseconds
    pub expires_at: Option<u64>, // Unix epoch in millisecondss
    pub starts_at: Option<u64>,
    pub updated_at: Option<u64>,
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    pub token_id: TokenId,
    pub copies_minted: u64,
    pub max_copies: u64,
    pub expires_at: Option<u64>,
    pub token_dependency_by_id: Vec<TokenId>,
    pub event_dependency_by_id: Vec<EventId>,
    pub account_approval_info_per_owner: LookupMap<AccountId, ApprovalInfo>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub token_dependency_by_id: Vec<TokenId>,
    pub event_dependency_by_id: Vec<EventId>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonTokenGeneral {
    pub token_id: TokenId,
    pub metadata: TokenMetadata,
    pub token_dependency_by_id: Vec<TokenId>,
    pub event_dependency_by_id: Vec<EventId>,
}

pub trait NonFungibleTokenMetadata {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

impl NFTContractMetadata {
    pub fn assert_valid_metadata(&self) {
        require!(self.reference_hash.0.len() == 32, "Hash has to be 32 bytes");
    }
}
