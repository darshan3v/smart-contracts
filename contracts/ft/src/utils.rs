use crate::Contract;

#[macro_export]
macro_rules! require {
    ( $a:expr, $b:expr ) => {
        if !$a {
            near_sdk::env::panic($b.as_bytes());
        }
    };
}

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            near_sdk::env::predecessor_account_id() == self.owner_id,
            "Only Owner of the Contract can call this method"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod test_utils {

    use crate::*;
    use near_sdk::json_types::Base64VecU8;
    use near_sdk::Balance;
    use near_sdk::VMContext;

    // Helper functions

    pub fn alice() -> ValidAccountId {
        ValidAccountId::try_from("alice.near").unwrap()
    }
    pub fn bob() -> ValidAccountId {
        ValidAccountId::try_from("bob.near").unwrap()
    }
    pub fn carol() -> ValidAccountId {
        ValidAccountId::try_from("carol.near").unwrap()
    }
    pub fn dex() -> ValidAccountId {
        ValidAccountId::try_from("dex.near").unwrap()
    }
    pub fn nft() -> ValidAccountId {
        ValidAccountId::try_from("nft.catchlabs.near").unwrap()
    }

    pub fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: "mike.near".to_string(),
            signer_account_id: predecessor_account_id.clone(),
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

    pub fn create_contract() -> Contract {
        let metadata = FungibleTokenMetadata {
            spec: String::from("1.1.0"),
            name: String::from("CAT Token"),
            symbol: String::from("CAT"),
            icon: String::from("C-A-T-C-H"),
            reference: String::from(
                "https://github.com/near/core-contracts/tree/master/w-near-141",
            ),
            reference_hash: Base64VecU8::from([5_u8; 32].to_vec()),
            decimals: 0,
        };
        let total_supply = U128::from(1_000_000_000_000_000);
        Contract::new(dex(), total_supply, metadata)
    }
}
