// will contain standalone NFT minting function

use crate::*;

#[near_bindgen]
impl Contract {
    pub fn is_eligible_to_mint_token(&self, account_id: AccountId, token_id: TokenId) -> bool {
        let token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"No Token"));

        self.internal_is_eligible_to_mint_token(&account_id, &token)
    }

    // General Purpose fn
    pub fn is_token_expired(&self, token_id: TokenId) -> bool {
        let token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"No Token"));

        internal_is_token_expired(&token)
    }

    // General Purpose fn
    pub fn can_token_be_minted(&self, token_id: TokenId) -> bool {
        let token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"No Token"));

        token.copies_minted < token.max_copies
    }
}
