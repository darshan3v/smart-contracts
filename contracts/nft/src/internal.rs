use crate::*;
use near_sdk::CryptoHash;
use std::mem::size_of;

//convert the royalty percentage and amount to pay into a payout (U128)
pub(crate) fn royalty_to_payout(royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / 10_000u128)
}

//calculate how many bytes the account ID is taking up
pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I, //the approved account IDs must be passed in as an iterator
) -> Promise
where
    I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids
        .map(bytes_for_approved_account_id)
        .sum();
    Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost())
}

pub(crate) fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    //call the refund_approved_account_ids_iter with the approved account IDs as keys
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

//used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

//Assert that the user has attached at least 1 yoctoNEAR (for security reasons and to pay for storage)
pub(crate) fn assert_at_least_one_yocto() {
    require!(
        env::attached_deposit() >= 1,
        "Requires attached deposit of at least 1 yoctoNEAR"
    );
}

//refund the initial deposit based on the amount of storage that was used up
pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    //make sure that the attached deposit is greater than or equal to the required cost
    require!(
        attached_deposit >= required_cost,
        format!("Must attach {} yoctoNEAR to cover storage", required_cost)
    );

    let refund = attached_deposit - required_cost;

    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

impl Contract {
    //add a token to the set of tokens an owner has
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        require!(
            is_catch_player(account_id.into()),
            "NFT's can only be Owned by Catch Players"
        );
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            //if the account doesn't have any tokens, we create a new unordered set
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInner {
                    //we get a new unique prefix for the collection
                    account_id_hash: hash_account_id(&account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        tokens_set.insert(token_id);

        //we insert that set for the given account ID.
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    //remove a token from an owner .
    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self
            .tokens_per_owner
            .get(account_id)
            .unwrap_or_else(|| env::panic(b"Token should be owned by the sender"));

        tokens_set.remove(token_id);

        //if the token set is now empty, we remove the owner from the tokens_per_owner collection
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            //if the token set is not empty, we simply insert it back for the account ID.
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    //internal method that transfers the NFT to the receiver_id
    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let token = self
            .tokens_by_id
            .get(token_id)
            .unwrap_or_else(|| env::panic(b"No token"));

        //if the sender doesn't equal the owner, we check if the sender is in the approval list
        if sender_id != &token.owner_id {
            require!(
                token.approved_account_ids.contains_key(sender_id),
                "Unauthorized"
            );
        }

        // If they included an approval_id, check if the sender's actual approval_id is the same as the one included
        if let Some(enforced_approval_id) = approval_id {
            //get the actual approval ID
            let actual_approval_id = token
                .approved_account_ids
                .get(sender_id)
                .unwrap_or_else(|| env::panic(b"Sender is not approved account"));

            require!(
                actual_approval_id == &enforced_approval_id,
                format!(
                    "The actual approval_id {} is different from the given approval_id {}",
                    actual_approval_id, enforced_approval_id
                )
            );
        }

        require!(
            &token.owner_id != receiver_id,
            "The token owner and the receiver should be different"
        );

        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        self.internal_add_token_to_owner(receiver_id, token_id);

        //we create a new token struct
        let new_token = Token {
            owner_id: receiver_id.clone(),
            //reset the approval account IDs
            approved_account_ids: Default::default(),
            next_approval_id: token.next_approval_id,
            //we copy over the royalties from the previous token
            royalty: token.royalty.clone(),
        };
        //insert that new token into the tokens_by_id, replacing the old entry
        self.tokens_by_id.insert(token_id, &new_token);

        //if there was some memo attached, we log it.
        if let Some(memo) = memo.as_ref() {
            env::log(format!("Memo: {}", memo).as_bytes());
        }

        // Default the authorized ID to be None for the logs.
        let mut authorized_id = None;
        //if the approval ID was provided, set the authorized ID equal to the sender
        if approval_id.is_some() {
            authorized_id = Some(sender_id.to_string());
        }

        let transfer_logs = vec![NftTransferLog {
            authorized_id,
            old_owner_id: token.owner_id.to_string(),
            new_owner_id: receiver_id.to_string(),
            token_ids: vec![token_id.to_string()],
            memo,
        }];
        NftTransferLog::emit(transfer_logs);
        token
    }
}
