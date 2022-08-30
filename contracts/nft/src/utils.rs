use crate::*;

#[macro_export]
macro_rules! require {
    ( $a:expr, $b:expr ) => {
        if !$a {
            near_sdk::env::panic($b.as_bytes());
        }
    };
}

impl Contract {
    /// Assert that Predecessor A/c is the Owner of the Contract
    pub fn assert_owner(&self) {
        require!(
            near_sdk::env::predecessor_account_id() == self.owner_id,
            "It is a owner only method"
        );
    }

    pub fn internal_is_eligible_to_mint_token(
        &self,
        account_id: &AccountId,
        token: &Token,
    ) -> bool {
        let has_dependency =
            token.token_dependency_by_id.len() > 0 || token.event_dependency_by_id.len() > 0;

        if has_dependency {
            let token_set = if let Some(token_set) = self.tokens_per_owner.get(&account_id) {
                token_set
            } else {
                return false;
            };

            if !token
                .token_dependency_by_id
                .iter()
                .all(|token_id| token_set.contains(&token_id))
            {
                return false;
            } else {
                let mut event_passes: Vec<TokenId>;

                for event_id in &token.event_dependency_by_id {
                    event_passes = if let Some(event) = self.events_by_id.get(&event_id) {
                        event.event_passes
                    } else {
                        return false;
                    };

                    if !event_passes
                        .iter()
                        .any(|token_id| token_set.contains(token_id))
                    {
                        return false;
                    }
                }
                true
            }
        } else {
            true
        }
    }
}

// TokenId and EventId can only contain Ascii Chars without . , it can contain a..=z , A..=Z, 0..=9, Space and Underscore and hyphen are also allowed but not fullstop
pub(crate) fn assert_valid_id(id: &str) {
    require!(
        id.bytes()
            .all(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b' ' | b'_' | b'-')),
        "Invalid TokenId / EventId"
    );
}

// Resolve token_id of form event_id.token_id.owner_id to Full TokenId and OwnerId
pub(crate) fn resolve_token_id(token_id: TokenId) -> (TokenId, AccountId) {
    let (event_id, token_id_and_owner_id) = token_id
        .split_once(".")
        .unwrap_or_else(|| env::panic(b"Invalid TokenId"));

    let (token_id, owner_id) = token_id_and_owner_id
        .split_once(".")
        .unwrap_or_else(|| env::panic(b"Invalid TokenId"));

    let token_id = format!("{}.{}", event_id, token_id);

    return (token_id, owner_id.to_string());
}

// Build token_id of form event_id.token_id.owner_id from TokenId and OwnerId
pub(crate) fn build_full_token_id(token_id: TokenId, owner_id: AccountId) -> TokenId {
    format!("{}.{}", token_id, owner_id)
}

// returns true if token has expired
pub(crate) fn internal_is_token_expired(token: &Token) -> bool {
    if let Some(t) = token.expires_at {
        if let Some(t) = t.checked_mul(1_000_000) {
            // Multiply by 1_000_000 to convert milli to nano seconds
            return t < env::block_timestamp();
        } else {
            env::panic(b"Time Stamp Overflow, Invalid ");
        }
    } else {
        return false;
    }
}

// panics if token can't be minted
pub(crate) fn assert_token_availability(token: &Token) {
    require!(
        token.copies_minted < token.max_copies,
        "All the copies of this token have been minted"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod test_utils {

    use crate::*;
    use near_sdk::json_types::ValidAccountId;
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
    pub fn nft() -> ValidAccountId {
        ValidAccountId::try_from("nft.catchlabs.near").unwrap()
    }
    pub fn marketplace() -> ValidAccountId {
        ValidAccountId::try_from("marketplace.near").unwrap()
    }

    pub fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: "nft.catchlabs.near".to_string(),
            signer_account_id: predecessor_account_id.clone(),
            signer_account_pk: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
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
        todo!()
    }
}

#[cfg(test)]
mod test_utility_fn {
    use super::*;
    use crate::utils::test_utils::*;
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    #[test]
    #[should_panic(expected = "Invalid TokenId / EventId")]
    fn panic_assert_valid_id() {
        testing_env!(get_context(carol().to_string(), 0));

        let id = String::from("eventid.tokenid");
        assert_valid_id(&id);
    }

    #[test]
    fn success_assert_valid_id() {
        testing_env!(get_context(carol().to_string(), 0));

        let id = String::from("Event_id 1 Token_id 2 - Owner_id");
        assert_valid_id(&id);
    }

    #[test]
    fn success_resolve_token_id() {
        testing_env!(get_context(carol().to_string(), 0));

        let token_id = String::from("event_id.token_id.owner.near");
        let expected_output = (
            String::from("event_id.token_id"),
            String::from("owner.near"),
        );
        assert_eq!(expected_output, resolve_token_id(token_id));
    }

    #[test]
    #[should_panic(expected = "Invalid TokenId")]
    fn panic_resolve_token_id() {
        testing_env!(get_context(carol().to_string(), 0));

        let token_id = String::from("event_id_token_id");
        resolve_token_id(token_id);
    }

    #[test]
    fn success_build_full_token_id() {
        testing_env!(get_context(carol().to_string(), 0));

        let token_id = String::from("event_id.token_id");
        let owner_id = String::from("owner_id");

        let full_token_id = String::from("event_id.token_id.owner_id");
        assert_eq!(full_token_id, build_full_token_id(token_id, owner_id));
    }
}
