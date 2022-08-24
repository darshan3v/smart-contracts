/**
 * Fungible Token NEP-141 Token contract
 *
 * The aim of the contract is to provide a basic implementation of the improved function token standard.
 *
 * lib.rs is the main entry point.
 * core_impl.rs implements NEP-141 standard
 * storage_impl.rs implements NEP-145 standard for allocating storage per account
 * ft_metadata.rs implements NEP-148 standard for providing token-specific metadata.
 * events.rs extends NEP-297 for better indexing
 * internal.rs contains internal methods for fungible token core.
 **/
mod core_impl;
mod events;
mod ft_metadata;
mod internal;
mod receiver;
mod resolver;
mod storage_impl;
mod utils;

pub use crate::core_impl::{FungibleToken, FungibleTokenCore};
pub use crate::events::{FtBurnLog, FtMintLog, FtTransferLog};
pub use crate::ft_metadata::FungibleTokenMetadata;
pub use crate::receiver::ext_fungible_token_receiver;
pub use crate::resolver::{ext_self, FungibleTokenResolver};
pub use crate::storage_impl::StorageManager;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, Gas,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult, StorageUsage,
};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

#[derive(BorshSerialize)]
pub enum StorageKey {
    Accounts,
    Metadata,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,

    //// Fungible Token
    pub token: FungibleToken,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Metadata for the Fungible Token
    pub ft_metadata: LazyOption<FungibleTokenMetadata>,
}

#[near_bindgen]
impl Contract {
    /// Initialize The Contract
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid_metadata();

        let owner_id: AccountId = owner_id.into();

        let token = FungibleToken {
            accounts: LookupMap::new(StorageKey::Accounts.try_to_vec().unwrap()),
            total_supply: total_supply.into(),
        };

        let ft_metadata =
            LazyOption::new(StorageKey::Metadata.try_to_vec().unwrap(), Some(&metadata));

        let mut this = Self {
            owner_id: owner_id.clone(),
            token,
            account_storage_usage: 0,
            ft_metadata,
        };
        // Determine cost of insertion into LookupMap

        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = unsafe { String::from_utf8_unchecked(vec![b'a'; 64]) };
        this.token.accounts.insert(&tmp_account_id, &0u128);
        this.account_storage_usage = env::storage_usage() - initial_storage_usage;
        this.token.accounts.remove(&tmp_account_id);

        // Make owner have total supply

        let total_supply_u128: u128 = total_supply.into();
        this.token.accounts.insert(&owner_id, &total_supply_u128);

        FtMintLog {
            owner_id: owner_id.to_string(),
            amount: total_supply,
            memo: Some("Mint".to_string()),
        }
        .emit();

        this
    }

    /// Transfer the Fungible Token from one A/c to another A/c
    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        self.token
            .ft_transfer(receiver_id.into(), amount.into(), memo)
    }

    /// Transfer Fungible tokens to a Contract and call on_transfer function of the contract
    /// returns the amount of tokens used by the contract
    #[payable]
    pub fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token
            .ft_transfer_call(receiver_id.into(), amount.into(), memo, msg)
    }

    /// Resolving Transaction after on_transfer is called on recieving contract
    ///
    /// Refunds and returns the unused tokens
    #[private]
    pub fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        // It is a Private Funciton and hence no need to check if the A/c is Catch A/c , It will always be valid
        self.token
            .ft_resolve_transfer(sender_id, receiver_id, amount)
    }

    /// Return total supply of Fungible Token
    pub fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    /// Return Fungible Token balance of the given A/c
    pub fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.token.ft_balance_of(account_id.into())
    }

}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod ft_core_tests {
    use super::*;
    use utils::test_utils::*;

    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    const ONE_YOCTO: Balance = 1;
    const STORAGE_COST: Balance = 1_250_000_000_000_000_000_000; // 1 Near = 10^24 Yocto Near

    // Test for new()

    #[test]
    fn contract_creation_with_new() {
        testing_env!(get_context(dex().to_string(), 0));

        let contract = create_contract();

        assert_eq!(contract.ft_total_supply().0, 1_000_000_000_000_000);
        assert_eq!(contract.ft_balance_of(alice()).0, 0);
        assert_eq!(contract.ft_balance_of(bob().into()).0, 0);
        assert_eq!(contract.ft_balance_of(carol().into()).0, 0);
        assert_eq!(
            contract.ft_balance_of(dex().into()).0,
            1_000_000_000_000_000
        );
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn default_fails() {
        testing_env!(get_context(carol().into(), 0));
        let _contract = Contract::default();
    }

    // Test for ft_transfer
    #[test]
    fn test_ft_transfer() {
        testing_env!(get_context(dex().to_string(), STORAGE_COST));

        let mut contract = create_contract();
        let amount = U128::from(100_000_000);
        let remaining_balance = U128::from(contract.ft_total_supply().0 - amount.0);

        contract.storage_deposit(Some(carol()));
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));
        contract.ft_transfer(carol(), amount, None);
        assert_eq!(contract.ft_balance_of(carol()), amount);
        assert_eq!(contract.ft_balance_of(dex()), remaining_balance);
    }

    #[test]
    #[should_panic(expected = "The account is not registered")]
    fn test_ft_transfer_fails() {
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));

        let mut contract = create_contract();
        let amount = U128::from(100_000_000);
        contract.ft_transfer(carol(), amount, None);
    }
}
