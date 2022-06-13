use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_NFT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;
const MIN_GAS_FOR_NFT_TRANSFER_CALL: Gas = 100_000_000_000_000;
const NO_DEPOSIT: Balance = 0;

pub trait NonFungibleTokenCore {
    //transfers an NFT to a receiver ID [transfers restricted only among Catch Players]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    //transfers an NFT to a receiver and calls a function on the receiver ID's contract
    /// Returns `true` if the token was transferred from the sender's account.
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    //get information about the NFT token passed in
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    //Method stored on the receiver contract that is called via cross contract call when nft_transfer_call is called
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    // resolves the promise of the cross contract call to the receiver contract
    fn nft_resolve_transfer(
        &mut self,
        //Authorized ID for logging the transfer event
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        //Approval map so we can keep track of what the approvals were before the transfer
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool;
}

trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool;
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

        //call the internal transfer method and get back the previous token so we can refund the approved account IDs
        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);

        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );
    }

    // ToDo -> Probably this functions not required and now still player can do proxy market my deploying contract on their account so have to think and will have to do like approval stuff
    // This will transfer the NFT and call a method on the reciver_id contract
    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();

        let attached_gas = env::prepaid_gas();

        require!(
            attached_gas >= MIN_GAS_FOR_NFT_TRANSFER_CALL,
            format!(
                "You cannot attach less than {:?} Gas to nft_transfer_call",
                MIN_GAS_FOR_NFT_TRANSFER_CALL
            )
        );
        let sender_id = env::predecessor_account_id();

        //transfer the token and get the previous token object
        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo.clone(),
        );

        let mut authorized_id = None;
        //if the sender isn't the owner of the token, we set the authorized ID equal to the sender.
        if sender_id != previous_token.owner_id {
            authorized_id = Some(sender_id.to_string());
        }

        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id,
            previous_token.owner_id.clone(),
            token_id.clone(),
            msg,
            &receiver_id,                                   //contract account
            NO_DEPOSIT,                                     //attached deposit
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL, //attached GAS
        )
        .then(ext_self::nft_resolve_transfer(
            authorized_id, // Authorized ID so that we can log the transfer
            previous_token.owner_id,
            receiver_id,
            token_id,
            previous_token.approved_account_ids,
            memo,
            &env::current_account_id(), //contract account
            NO_DEPOSIT,                 //attached deposit
            GAS_FOR_RESOLVE_TRANSFER,   //GAS attached to the call
        ))
        .into()
    }

    //get the information for a specific token ID
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonToken {
                token_id,
                owner_id: token.owner_id,
                metadata,
                approved_account_ids: token.approved_account_ids,
                royalty: token.royalty,
            })
        } else {
            //if there wasn't a token ID in the tokens_by_id collection, we return None
            None
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    //resolves the cross contract call when calling nft_on_transfer in the nft_transfer_call method
    //returns true if the token was successfully transferred to the receiver_id
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool {
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                if !return_token {
                    refund_approved_account_ids(owner_id, &approved_account_ids);
                    return true;
                }
            }
        }

        //get the token object if there is some token object
        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                refund_approved_account_ids(owner_id, &approved_account_ids);
                return true;
            }
            token
        //if there isn't a token object, it was burned and so we return true
        } else {
            refund_approved_account_ids(owner_id, &approved_account_ids);
            return true;
        };

        self.internal_remove_token_from_owner(&receiver_id, &token_id);
        self.internal_add_token_to_owner(&owner_id, &token_id);

        token.owner_id = owner_id.clone();

        //we refund the receiver any approved account IDs that they may have set on the token
        refund_approved_account_ids(receiver_id.clone(), &token.approved_account_ids);
        //reset the approved account IDs to what they were before the transfer
        token.approved_account_ids = approved_account_ids;

        //we inset the token back into the tokens_by_id collection
        self.tokens_by_id.insert(&token_id, &token);

        let transfer_logs = vec![NftTransferLog {
            authorized_id,
            old_owner_id: receiver_id.to_string(),
            new_owner_id: owner_id.to_string(),
            token_ids: vec![token_id.to_string()],
            memo,
        }];

        //Emitting Reverting Log
        NftTransferLog::emit(transfer_logs);
        false
    }
}
