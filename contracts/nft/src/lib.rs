/**
* Non Fungible Token NEP-171 Token contract
* Approval NEP-178
* Enumeration NEP-181
* Metadata NEP-177
* Royalties and Payout NEP-199
*
* The aim of the contract is to provide a basic implementation of the improved function NFT standard.
*
* lib.rs is the main entry point.
* nft_core.rs implements NEP-171 standard handles core function regarding nft transfers [restricted to  only catch sub account's that is players]
* approval.rs implements Approval Management NEP-178 for management of approvals of transfer of NFT and   also implements Marketplace Approval System.
* enumeration.rs implements NEP-181 standard for getter functions to retrieve data off-chain
* mint.rs implements nft_minting functionality
* royalty.rs implements functionality regarding perpetual royalties and payout
* metadata.rs implements NEP-177 standard for both Contract and NFT-specific metadata.
* events.rs extends NEP-297 for better indexing
* internal.rs contains internal methods.
**/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, CryptoHash, Gas,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult,
};
use std::collections::HashMap;

pub use crate::approval::*;
pub use crate::events::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::NonFungibleTokenCore;
pub use crate::royalty::*;
use crate::utils::is_catch_player;

mod approval;
mod enumeration;
mod events;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalty;
mod utils;

const CATCH_MARKETPLACE_CONTRACT: &str = "marketplace.catchlabs.near";

const CATCH_MARKETPLACE_CONTRACT_TESTNET: &str = "marketplace.catchlabs.testnet";

#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    ApprovedMarketplaces,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the approved marketplace contracts
    pub approved_marketplaces: UnorderedSet<AccountId>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,
}

#[near_bindgen]
impl Contract {
    /// Initialize The Contract
    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        metadata.assert_valid_metadata();
        let mut this = Self {
            owner_id: owner_id.into(),
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            approved_marketplaces: UnorderedSet::new(
                StorageKey::ApprovedMarketplaces.try_to_vec().unwrap(),
            ),
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
        };

        let catch_marketplace = AccountId::from(CATCH_MARKETPLACE_CONTRACT_TESTNET);

        this.approved_marketplaces.insert(&catch_marketplace);

        this
    }

    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        //calls the other function "new: with some default metadata and the owner_id passed in
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Catch NFT Contract".to_string(),
                symbol: "CATCH".to_string(),
                icon: None,
                base_uri: "ipfs".to_string(),
                reference: "ipfs://example.com/hash".to_string(),
                reference_hash: Base64VecU8::from([5_u8; 32].to_vec()),
            },
        )
    }
}
