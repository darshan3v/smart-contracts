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
}
