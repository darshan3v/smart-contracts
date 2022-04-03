/**
* Fungible Token NEP-141 Token contract
*
* The aim of the contract is to provide a basic implementation of the improved function token standard.
*
* lib.rs is the main entry point.
* core_impl.rs implements NEP-141 standard
* storage_impl.rs implements NEP-145 standard for allocating storage per account
* metadata.rs implements NEP-148 standard for providing token-specific metadata.
* internal.rs contains internal methods for fungible token core.
*/
mod core_impl;
mod internal;
mod metadata;
mod receiver;
mod resolver;
mod storage_impl;
mod users_impl;
mod utils;

pub use crate::core_impl::*;
pub use crate::metadata::*;
pub use crate::receiver::*;
pub use crate::resolver::*;
pub use crate::storage_impl::*;
pub use crate::users_impl::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, StorageUsage,
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
        let mut this = Self {
            owner_id: owner_id.clone(),
            token,
            account_storage_usage: 0,
            ft_metadata: LazyOption::new(
                StorageKey::Metadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
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
        this
    }

    /// Transfer the Fungible Token from one A/c to another A/c
    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        require!(
            self.token
                .accounts
                .contains_key(&env::predecessor_account_id()),
            "Register for Catch A/c"
        );

        require!(
            self.token.accounts.contains_key(&receiver_id),
            "Register for Catch A/c"
        );
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    /// Transfer Fungible tokens to a Contract and call on_transfer function of the contract
    #[payable]
    pub fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(
            self.token
                .accounts
                .contains_key(&env::predecessor_account_id()),
            "Register for Catch A/c"
        );

        require!(
            self.token.accounts.contains_key(&receiver_id),
            "Register for Catch A/c"
        );

        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    /// Resolving Transaction after on_transfer is called on recieving contract
    ///
    /// Refunds and returns the unused tokens
    /// Private fn
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

    /// Mint Fungible Token to the Owner A/c
    pub fn mint(&mut self, amount: U128) {
        self.assert_owner(); // Only owner can call

        let amount: Balance = amount.into();
        let owner_id = self.owner_id.clone();

        if let Some(new_total_supply) = self.token.total_supply.checked_add(amount) {
            self.token.total_supply = new_total_supply;
        } else {
            env::panic(b"Total Supply Overflow");
        }

        self.token.internal_deposit(&owner_id, amount);

        // ToDo - Mint Event
    }

    /// Transfer Fungible Token Rewards to players
    pub fn ft_transfer_player_reward(
        &mut self,
        player_id: AccountId,
        amount: U128,
        feat: Option<String>,
    ) {
        self.assert_owner();
        require!(
            self.token.accounts.contains_key(&player_id),
            "Register for Catch A/c"
        );

        let amount: Balance = amount.into();

        require!(amount > 0, "The amount should be a positive number");

        let owner_id = self.owner_id.clone();
        let player_id: AccountId = player_id.into();

        self.token.internal_withdraw(&owner_id, amount);
        self.token.internal_deposit(&player_id, amount);

        // ToDo - Transfer Reward Event
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod fungible_token_tests {
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use super::*;
    use near_sdk::json_types::Base64VecU8;

    const ZERO_U128: Balance = 0u128;

    // Helper functions

    fn alice() -> ValidAccountId {
        ValidAccountId::try_from("alice.near").unwrap()
    }
    fn bob() -> ValidAccountId {
        ValidAccountId::try_from("bob.near").unwrap()
    }
    fn carol() -> ValidAccountId {
        ValidAccountId::try_from("carol.near").unwrap()
    }
    fn dex() -> ValidAccountId {
        ValidAccountId::try_from("dex.near").unwrap()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: "mike.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    fn create_contract() -> Contract {
        let metadata = FungibleTokenMetadata {
            spec: String::from("1.1.0"),
            name: String::from("CAT Token"),
            symbol: String::from("CAT"),
            icon: Some(String::from("C-A-T-C-H")),
            reference: Some(String::from(
                "https://github.com/near/core-contracts/tree/master/w-near-141",
            )),
            reference_hash: Some(Base64VecU8::from([5_u8; 32].to_vec())),
            decimals: 0,
        };
        let total_supply = U128::from(1_000_000_000_000_000);
        Contract::new(dex(), total_supply, metadata)
    }

    // Test for new()

    #[test]
    fn contract_creation_with_new() {
        testing_env!(get_context(dex().to_string()));

        let contract = create_contract();

        assert_eq!(contract.ft_total_supply().0, 1_000_000_000_000_000);
        assert_eq!(contract.ft_balance_of(alice()).0, ZERO_U128);
        assert_eq!(contract.ft_balance_of(bob().into()).0, ZERO_U128);
        assert_eq!(contract.ft_balance_of(carol().into()).0, ZERO_U128);
        assert_eq!(
            contract.ft_balance_of(dex().into()).0,
            1_000_000_000_000_000
        );
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn default_fails() {
        testing_env!(get_context(carol().into()));
        let _contract = Contract::default();
    }

    // Test for mint()

    #[test]
    fn test_mint_success() {
        testing_env!(get_context(dex().to_string()));

        let mut contract = create_contract();
        contract.mint(U128::from(5));

        assert_eq!(contract.ft_total_supply().0, 1_000_000_000_000_005);
        assert_eq!(
            contract.ft_balance_of(dex().into()).0,
            1_000_000_000_000_005
        );
    }

    #[test]
    #[should_panic(expected = "It is a owner only method")]
    fn test_mint_fail() {
        testing_env!(get_context(alice().to_string()));
        let mut contract = create_contract();
        contract.mint(U128::from(5));
    }

    // Todo -> Test for ft_transfer_player_reward
}
