use crate::*;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::Serialize;
use near_sdk::{assert_one_yocto, env, log, AccountId, Balance, Promise};

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    total: U128,
    available: U128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

pub trait StorageManager {
    /// Wallet UX Security -> Attach 1 Yocto,
    ///
    /// Removes the A/c if no tokens present, burns token only if force = true
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

    /// Returns min and max NEAR that can be deposited for storage,
    ///
    /// Here min = max
    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    /// Returns Storage Balance of a given A/c,here it's the Same for Every Registered A/c,
    ///
    /// None is returned for Unregistered A/c
    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance>;
}

/************************************************/
/*  IMPLEMENTING STORAGE MANAGER FUNCTIONALITY  */
/************************************************/

#[near_bindgen]
impl StorageManager for Contract {
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&(account_id.into()))
    }
}

/*********************************************/
/*  INTERNAL FUNCTIONS - STORAGE MANAGEMENT  */
/*********************************************/

impl Contract {
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        if let Some(balance) = self.token.accounts.get(&account_id) {
            if balance == 0 || force {
                self.token.accounts.remove(&account_id);
                // no need to check as balance subtracted will always be valid
                self.token.total_supply -= balance;

                // ToDo -> Emit Burn Event

                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
                log!(
                    "{} sucessfully removed and {} remaining tokens burnt",
                    &account_id,
                    balance
                );
                Some((account_id, balance))
            } else {
                env::panic(b"Can't unregister the account with the positive balance without force")
            }
        } else {
            log!("The account {} is not registered", &account_id);
            None
        }
    }

    pub fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.token.accounts.contains_key(account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}
