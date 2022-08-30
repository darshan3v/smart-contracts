use crate::*;

const GAS_FOR_NFT_APPROVE: Gas = 10_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

#[derive(BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct ApprovalInfo {
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub next_approval_id: u64,
}

pub trait NonFungibleTokenCore {
    //approve an account ID to transfer a token on your behalf, here it will be only for marketplaces
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

    //check if the passed in account has access to approve the token ID
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;

    //revoke a specific account from transferring the token on your behalf
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    //revoke all accounts from transferring the token on your behalf
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleTokenApprovalsReceiver {
    //cross contract call to an external contract that is initiated during nft_approve
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
        assert_at_least_one_yocto();

        let (token_id, owner_id) = resolve_token_id(token_id);

        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        require!(
            self.approved_marketplaces.contains(&account_id),
            "You cannot list on other marketplaces other than Catch Approved Marketplace"
        );

        require!(
            !internal_is_token_expired(&token),
            "Can't approve other account since Token has already expired"
        );

        let valid_token_owner = if let Some(token_set) = self.tokens_per_owner.get(&owner_id) {
            token_set.contains(&token_id)
        } else {
            false
        };

        if !valid_token_owner {
            env::panic(b"Only Token owner can approve other accounts");
        }

        require!(
            &env::predecessor_account_id() == &owner_id,
            "Predecessor must be the token owner."
        );

        let mut approval_info = token
            .account_approval_info_per_owner
            .get(&owner_id)
            .unwrap_or_default();

        let approval_id: u64 = approval_info.next_approval_id;

        let is_new_approval = approval_info
            .approved_account_ids
            .insert(account_id.clone(), approval_id)
            .is_none();

        //if it was a new approval, we need to calculate how much storage is being used to add the account.
        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&account_id)
        //if it was not a new approval, we used no storage.
        } else {
            0
        };

        approval_info.next_approval_id += 1;

        token
            .account_approval_info_per_owner
            .insert(&owner_id, &approval_info);

        self.tokens_by_id.insert(&token_id, &token);

        refund_deposit(storage_used);

        //if some message was passed into the function, we initiate a cross contract call on the
        //account we're giving access to.
        if let Some(msg) = msg {
            ext_non_fungible_approval_receiver::nft_on_approve(
                build_full_token_id(token_id, owner_id.clone()),
                owner_id,
                approval_id,
                msg,
                &account_id, // contract account we're calling
                NO_DEPOSIT,  // NEAR deposit we attach to the call
                env::prepaid_gas() - GAS_FOR_NFT_APPROVE, // GAS we're attaching
            )
            .as_return(); // Returning this promise
        }
    }

    //check if the passed in account has access to approve the token ID
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        let (token_id, owner_id) = resolve_token_id(token_id);

        let token = self
            .tokens_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic(b"No token"));

        let approved_account_ids =
            if let Some(approval_info) = token.account_approval_info_per_owner.get(&owner_id) {
                approval_info.approved_account_ids
            } else {
                return false;
            };

        let approval = approved_account_ids.get(&approved_account_id);

        //if there was some approval ID found for the account ID
        if let Some(approval) = approval {
            //if a specific approval_id was passed into the function
            if let Some(approval_id) = approval_id {
                //return if the approval ID passed in matches the actual approval ID for the account
                approval_id == *approval
                //if there was no approval_id passed into the function, we simply return true
            } else {
                true
            }
            //if there was no approval ID found for the account ID, we simply return false
        } else {
            false
        }
    }

    //revoke a specific account from transferring the token on your behalf
    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();

        let (token_id, owner_id) = resolve_token_id(token_id);

        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        let predecessor_account_id = env::predecessor_account_id();

        let valid_token_owner = if let Some(token_set) = self.tokens_per_owner.get(&owner_id) {
            token_set.contains(&token_id)
        } else {
            false
        };

        if !valid_token_owner {
            env::panic(b"Only NFT owner can revoke access");
        }

        require!(
            &predecessor_account_id == &owner_id,
            "Revoke can only be performed by owner of NFT"
        );

        if let Some(mut approval_info) = token.account_approval_info_per_owner.get(&owner_id) {
            if approval_info
                .approved_account_ids
                .remove(&account_id)
                .is_some()
            {
                refund_approved_account_ids_iter(predecessor_account_id, [account_id].iter());

                token
                    .account_approval_info_per_owner
                    .insert(&owner_id, &approval_info);

                self.tokens_by_id.insert(&token_id, &token);
            }
        }
    }

    //revoke all accounts from transferring the token on your behalf
    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let (token_id, owner_id) = resolve_token_id(token_id);

        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        let predecessor_account_id = env::predecessor_account_id();

        let valid_token_owner = if let Some(token_set) = self.tokens_per_owner.get(&owner_id) {
            token_set.contains(&token_id)
        } else {
            false
        };

        if !valid_token_owner {
            env::panic(b"Only NFT owner can revoke access");
        }

        require!(
            &predecessor_account_id == &owner_id,
            "Revoke can only be performed by owner of NFT"
        );

        let (mut approved_account_ids, next_approval_id) =
            if let Some(approval_info) = token.account_approval_info_per_owner.get(&owner_id) {
                (
                    approval_info.approved_account_ids,
                    approval_info.next_approval_id,
                )
            } else {
                return ();
            };

        //only revoke if the approved account IDs for the token is not empty
        if !approved_account_ids.is_empty() {
            refund_approved_account_ids(predecessor_account_id, &approved_account_ids);
            approved_account_ids.clear();

            token.account_approval_info_per_owner.insert(
                &owner_id,
                &ApprovalInfo {
                    approved_account_ids,
                    next_approval_id,
                },
            );

            self.tokens_by_id.insert(&token_id, &token);
        }
    }
}

#[near_bindgen]
impl Contract {
    // This function adds marketplaces to the approved marketplace list allowing users to list their
    // NFT's on this marketplace
    // [Doing it based on assumption for Optimisation reasons]
    // Also this assumes that there will be enough Near for storage in the contract, this can be ensured and even panic won't cause any issues
    pub fn approve_marketplaces(&mut self, marketplaces: Vec<ValidAccountId>) -> Vec<bool> {
        self.assert_owner();

        let mut added = Vec::with_capacity(marketplaces.len());

        for marketplace in marketplaces {
            added.push(self.approved_marketplaces.insert(&marketplace.into()));
        }

        added
    }
}
