use crate::utils::is_valid_prefix;
use crate::*;
use near_sdk::{
    assert_self, ext_contract, log, near_bindgen, AccountId, Balance, Gas, Promise, PromiseResult,
    PublicKey,
};

// ToDo -> Yet to Calculate
const GAS_FOR_ACC_CREATION: Gas = 5_000_000_000_000;
const GAS_FOR_CALLBACK: Gas = 5_000_000_000_000;

#[ext_contract(ext_self)]
pub trait AccCreationExt {
    /// Register A/c
    ///
    /// Returns The Storage Balance
    fn acc_creation_callback(&mut self, account_id: AccountId) -> StorageBalance;
}

pub trait AccCreation {
    fn acc_creation_callback(&mut self, account_id: AccountId) -> StorageBalance;
}

impl AccCreation for Contract {
    fn acc_creation_callback(&mut self, account_id: AccountId) -> StorageBalance {
        assert_self(); // It is a private function

        require!(env::promise_results_count() == 1, "ERR_TOO_MANY_RESULTS");

        let amount: Balance = env::attached_deposit();

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"Account Creation Failed"),
            PromiseResult::Successful(_) => {
                if self.token.accounts.contains_key(&account_id) {
                    log!("The account is already registered");
                } else {
                    let min_balance = self.storage_balance_bounds().min.0;

                    require!(
                        !amount < min_balance,
                        "The attached deposit is less than the minimum storage balance"
                    );

                    self.token.accounts.insert(&account_id, &0_u128);
                }
                self.internal_storage_balance_of(&account_id).unwrap()
            }
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Creates Sub A/c of type player_name.catchlabs.near -> here catchlabs is contract name
    pub fn create_sub_account(
        &mut self,
        prefix: AccountId,
        player_public_key: PublicKey,
    ) -> Promise {
        self.assert_owner();

        require!(
            env::prepaid_gas() > GAS_FOR_ACC_CREATION + GAS_FOR_CALLBACK,
            "More Gas is Required"
        );

        require!(is_valid_prefix(&prefix), "Invalid Player Name");

        let storage_cost: Balance = self.storage_balance_bounds().min.0;
        let subaccount_id = AccountId::from(format!("{}.{}", prefix, env::current_account_id()));

        Promise::new(subaccount_id.clone())
            .create_account()
            .add_full_access_key(player_public_key)
            .then(ext_self::acc_creation_callback(
                subaccount_id,
                &env::current_account_id(), // Target Contract
                storage_cost,
                env::prepaid_gas() - GAS_FOR_ACC_CREATION,
            ))
    }
}
