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
    /// Deposit Near for the purpose of storage costs
    ///
    /// If Owner is calling contract funds will be directly used for the storage costs
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> StorageBalance;

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
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> StorageBalance {
        let amount: Balance = env::attached_deposit();

        let account_id: AccountId = match account_id {
            Some(acc_id) => acc_id.into(),
            None => env::predecessor_account_id().into(),
        };

        if self.token.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;

            // If owner called this then directly use contract funds
            if amount < min_balance && env::predecessor_account_id() != self.owner_id {
                env::panic(b"The attached deposit is less than the minimum storage balance");
            }

            // Checking if attached deposit + contract funds are enough to cover storage costs for new_user
            require!(
                env::account_balance() > min_balance,
                "Not Enough funds to cover storage costs for the user"
            );

            self.token.accounts.insert(&account_id, &0);

            let refund = amount.checked_sub(min_balance).unwrap_or_else(|| 0);

            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

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

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod fungible_token_tests {
    use super::*;
    use near_sdk::json_types::Base64VecU8;
    use near_sdk::Balance;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const ONE_YOCTO: Balance = 1;

    // Helper functions
    fn carol() -> ValidAccountId {
        ValidAccountId::try_from("carol.near").unwrap()
    }
    fn dex() -> ValidAccountId {
        ValidAccountId::try_from("dex.near").unwrap()
    }

    fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
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
            attached_deposit,
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
            reference: String::from(
                "https://github.com/near/core-contracts/tree/master/w-near-141",
            ),
            reference_hash: Base64VecU8::from([5_u8; 32].to_vec()),
            decimals: 0,
        };
        let total_supply = U128::from(1_000_000_000_000_000);
        Contract::new(dex(), total_supply, metadata)
    }

    #[test]
    #[should_panic(expected = "The attached deposit is less than the minimum storage balance")]
    fn storage_deposit_fails() {
        testing_env!(get_context(carol().to_string(), 500));
        let mut contract = create_contract();
        contract.storage_deposit(Some(carol()));
    }

    #[test]
    #[should_panic(
        expected = "Can't unregister the account with the positive balance without force"
    )]
    fn storage_unregister_fails() {
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));
        let mut contract = create_contract();
        contract.storage_deposit(Some(carol()));
        contract.ft_transfer(carol(), U128::from(1000), None);

        testing_env!(get_context(carol().to_string(), ONE_YOCTO));
        contract.storage_unregister(Some(false));
    }

    #[test]
    fn storage_unregister() {
        testing_env!(get_context(carol().to_string(), ONE_YOCTO));
        let mut contract = create_contract();
        // can't unregister A/c bcz not registered itself
        assert!(!contract.storage_unregister(Some(false)));

        testing_env!(get_context(dex().to_string(), ONE_YOCTO));

        contract.storage_deposit(Some(carol()));
        contract.ft_transfer(carol(), U128::from(1000), None);

        testing_env!(get_context(carol().to_string(), ONE_YOCTO));

        let remaining_balance = contract.ft_total_supply().0 - 1000;

        // can unregister A/c with force the balance is burnt
        assert!(contract.storage_unregister(Some(true)));
        assert_eq!(contract.ft_balance_of(dex()).0, remaining_balance);
        assert_eq!(contract.ft_balance_of(carol()).0, 0);
        assert!(!contract.storage_balance_of(carol()).is_some());
    }
}
