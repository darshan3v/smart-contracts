use crate::*;

#[near_bindgen]
impl Contract {
    //Query for the total supply of NFTs on the contract here it is type of passes issued
    pub fn nft_total_supply(&self) -> U128 {
        U128(self.token_metadata_by_id.len() as u128)
    }

    //Query for nft tokens on the contract regardless of the owner using pagination [Common Data of all passes]
    pub fn nft_tokens(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonTokenGeneral> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.token_metadata_by_id
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|token_id| self.nft_token_general(token_id.clone()).unwrap())
            .collect()
    }

    //get the total supply of NFTs for a given owner
    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);

        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            U128(tokens_for_owner_set.len() as u128)
        } else {
            U128(0)
        }
    }

    //Query for all the tokens for an owner
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonToken> {
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        let tokens = if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            tokens_for_owner_set
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));

        tokens
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|token_id| {
                self.nft_token(build_full_token_id(token_id, account_id.clone()))
                    .unwrap()
            })
            .collect()
    }

    //Query for all the tokens for an owner
    pub fn get_events(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<(EventId, Event)> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.events_by_id
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|a| a)
            .collect()
    }

    //Returns paginated view of approved marketplaces
    pub fn get_approved_marketplace(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<AccountId> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.approved_marketplaces
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|marketplace_id| marketplace_id)
            .collect()
    }
}

impl Contract {
    fn nft_token_general(&self, token_id: TokenId) -> Option<JsonTokenGeneral> {
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonTokenGeneral {
                token_id,
                metadata,
                token_dependency_by_id: token.token_dependency_by_id,
                event_dependency_by_id: token.event_dependency_by_id,
            })
        } else {
            //if there wasn't a token ID in the tokens_by_id collection, we return None
            None
        }
    }
}
