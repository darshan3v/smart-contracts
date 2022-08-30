use crate::*;

pub type EventId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Event {
    pub organiser: AccountId,
    pub event_passes: Vec<TokenId>,
    // pub event_metadata: EventMetadata     Will be included in Future version of contract
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenInfo {
    pub token_id: TokenId,
    pub token_metadata: TokenMetadata,
    pub token_dependency_by_id: Vec<TokenId>,
    pub event_dependency_by_id: Vec<EventId>,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn organise_event(&mut self, event_id: EventId, tokens: Vec<TokenInfo>) {
        let initial_storage = env::storage_usage();

        let mut event_passes: Vec<TokenId> = Vec::with_capacity(tokens.len());
        let mut token_id: TokenId;
        let mut token: Token;
        let mut storage_required_for_token_ids = 0;
        let event: Event;
        let organiser = env::predecessor_account_id();

        assert_valid_id(&event_id);

        for token_info in tokens {
            assert_valid_id(&token_info.token_id);

            token_id = format!("{}.{}", event_id, token_info.token_id); // TokenId = EventId.TokenId

            event_passes.push(token_id.clone());

            token = Token {
                token_id: token_id.clone(),
                copies_minted: 0,
                max_copies: token_info.token_metadata.copies.unwrap_or_else(|| 1),
                expires_at: token_info.token_metadata.expires_at,
                token_dependency_by_id: token_info.token_dependency_by_id,
                event_dependency_by_id: token_info.event_dependency_by_id,
                account_approval_info_per_owner: LookupMap::new(
                    StorageKey::ApprovedAccountsPerToken {
                        token_id_hash: hash_id(&token_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                ),
            };

            require!(
                self.tokens_by_id.insert(&token_id, &token).is_none(),
                "Token Already exists"
            );

            self.token_metadata_by_id
                .insert(&token_id, &token_info.token_metadata);

            storage_required_for_token_ids +=
                bytes_for_token_or_event_or_account_id(&token_id) * token.max_copies;
        }

        assert_valid_id(&event_id);

        event = Event {
            organiser,
            event_passes,
        };

        require!(
            self.events_by_id.insert(&event_id, &event).is_none(),
            "Event Already Exists"
        );

        let total_storage_required =
            env::storage_usage() - initial_storage + storage_required_for_token_ids;

        refund_deposit(total_storage_required);
    }

    #[payable]
    pub fn nft_event_register(&mut self, receiver_id: AccountId, token_id: TokenId) {
        let mut token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"Token does not exist"));

        let account_id = receiver_id;

        require!(!internal_is_token_expired(&token), "Token has expired");

        assert_token_availability(&token);

        require!(
            self.internal_is_eligible_to_mint_token(&account_id, &token),
            format!(
                "{} doesn't satisfy all the dependencies for the token {}",
                &account_id, &token_id
            )
        );

        token.copies_minted += 1;

        self.tokens_by_id.insert(&token_id, &token);

        if self.tokens_per_owner.get(&account_id).is_none() {
            let storage_used = bytes_for_token_or_event_or_account_id(&account_id);
            refund_deposit(storage_used);
        }

        self.internal_add_token_to_owner(&account_id, &token_id);

        let event_id: EventId = token_id.split_once(".").unwrap().0.to_string();

        NftMintLog::emit(vec![NftMintLog {
            owner_id: account_id.clone(),
            token_id: vec![token_id],
            memo: Some(format!(
                "{} has successfully registered for the event {}",
                &account_id, &event_id
            )),
        }]);

        todo!(); // Refund User if payed extra
    }
}
