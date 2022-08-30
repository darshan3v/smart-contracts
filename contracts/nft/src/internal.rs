use crate::*;

//calculate how many bytes the account ID is taking up
pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn bytes_for_token_or_event_or_account_id(id: &String) -> u64 {
    id.as_str().len() as u64 + 4
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
pub(crate) fn hash_id(id: &str) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(id.as_bytes()));
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
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            //if the account doesn't have any tokens, we create a new unordered set
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInner {
                    //we get a new unique prefix for the collection
                    account_id_hash: hash_id(&account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        require!(
            tokens_set.insert(token_id),
            format!("{} account already has token {}", &account_id, &token_id)
        );

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
    ) -> (AccountId, ApprovalInfo) {
        let (token_id, owner_id) = resolve_token_id(token_id.to_string());

        let mut token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"No token"));

        require!(
            !internal_is_token_expired(&token),
            "Token Can't be transferred Since it has already expired"
        );

        let token_set = self
            .tokens_per_owner
            .get(&owner_id)
            .unwrap_or_else(|| env::panic(b"You own no tokens"));

        require!(
            token_set.contains(&token_id),
            "You need to own the token to transfer it"
        );

        require!(
            self.internal_is_eligible_to_mint_token(receiver_id, &token),
            "receiver_id doesn't satisfy all dependencies for the token"
        );

        require!(
            &owner_id != receiver_id,
            "The token owner and the receiver should be different"
        );

        let (mut old_approval_info, mut approval_info) = Default::default();

        //if the sender doesn't equal the owner, we check if the sender is in the approval list

        if sender_id != &owner_id {
            approval_info = token
                .account_approval_info_per_owner
                .get(&owner_id)
                .unwrap_or_else(|| env::panic(b"Token Owner hasn't approved any account"));

            require!(
                approval_info.approved_account_ids.contains_key(sender_id),
                "Unauthorised"
            );

            old_approval_info = approval_info.clone();

            // If they included an approval_id, check if the sender's actual approval_id is the same as the one included

            if let Some(enforced_approval_id) = approval_id {
                //get the actual approval ID

                let actual_approval_id = approval_info
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
        }

        // update token struct
        token.account_approval_info_per_owner.remove(&owner_id);

        approval_info.approved_account_ids = Default::default();
        token
            .account_approval_info_per_owner
            .insert(receiver_id, &approval_info);

        //insert that new token into the tokens_by_id, replacing the old entry
        self.tokens_by_id.insert(&token_id, &token);

        self.internal_remove_token_from_owner(&owner_id, &token_id);
        self.internal_add_token_to_owner(receiver_id, &token_id);

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
            old_owner_id: owner_id.clone(),
            new_owner_id: receiver_id.to_string(),
            token_id: vec![token_id],
            memo,
        }];

        NftTransferLog::emit(transfer_logs);

        (owner_id, old_approval_info)
    }
}
