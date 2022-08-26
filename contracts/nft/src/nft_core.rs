use crate::*;

pub trait NonFungibleTokenCore {
    //transfers an NFT to a receiver ID (if eligible) and returns Payout Object
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    //transfers an NFT to a receiver ID (if eligible) and returns Payout Object
    //need to take max_len_payout as argument for compatibility purpose
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        balance: U128,
        memo: Option<String>,
        max_len_payout: u32,
    ) -> Payout;

    //get information about the NFT token passed in
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    //This transfers the NFT from the current owner to the receiver.
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        //call the internal transfer method and get back the previous token owner and approved_ids
        let (old_owner_id, old_approval_info) =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);

        refund_approved_account_ids(old_owner_id, &old_approval_info.approved_account_ids);
    }

    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        balance: U128,
        memo: Option<String>,
        max_len_payout: u32,
    ) -> Payout {
        self.nft_transfer(receiver_id, token_id.clone(), approval_id, memo);

        let (_, owner_id) = resolve_token_id(token_id);

        Payout {
            payout: HashMap::from([(owner_id, balance)]),
        }
    }

    //get the information for a specific token ID
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        let (token_id, owner_id) = resolve_token_id(token_id);
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonToken {
                token_id: build_full_token_id(token_id, owner_id.clone()),
                owner_id: owner_id.clone(),
                metadata,
                approved_account_ids: token
                    .account_approval_info_per_owner
                    .get(&owner_id)
                    .unwrap_or_default()
                    .approved_account_ids,
                token_dependency_by_id: token.token_dependency_by_id,
                event_dependency_by_id: token.event_dependency_by_id,
            })
        } else {
            //if there wasn't a token ID in the tokens_by_id collection, we return None
            None
        }
    }
}
