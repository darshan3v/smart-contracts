use crate::*;

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
            require!(
                amount >= min_balance,
                format!("Please Attach a deposit of {} Yocto Near", min_balance)
            );

            self.token.accounts.insert(&account_id, &0);

            let refund = amount - min_balance;

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

                FtBurnLog {
                    owner_id: account_id.to_string(),
                    amount: U128::from(balance),
                    memo: Some("Account Unregistered ! & Tokens burnt if there".to_string()),
                }
                .emit();

                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
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
mod storage_tests {
    use super::*;
    use utils::test_utils::*;

    use near_sdk::testing_env;
    use near_sdk::Balance;
    use near_sdk::MockedBlockchain;

    const ONE_YOCTO: Balance = 1;
    const STORAGE_COST: Balance = 1_250_000_000_000_000_000_000; // 1 Near = 10^24 Yocto Near

    #[test]
    #[should_panic(expected = "Please Attach a deposit of 1250000000000000000000 Yocto Near")]
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
        testing_env!(get_context(dex().to_string(), STORAGE_COST));
        let mut contract = create_contract();
        contract.storage_deposit(Some(carol()));
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));
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

        testing_env!(get_context(dex().to_string(), STORAGE_COST));

        contract.storage_deposit(Some(carol()));
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));
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
