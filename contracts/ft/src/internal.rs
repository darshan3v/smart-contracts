use crate::*;

/**********************************************/
/*  INTERNAL FUNCTIONS - FUNGIBLE TOKEN CORE  */
/**********************************************/

impl FungibleToken {
    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self
            .accounts
            .get(&account_id)
            .unwrap_or_else(|| env::panic(b"The account is not registered"));

        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(&account_id, &new_balance);
        } else {
            env::panic(b"Balance overflow");
        }
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self
            .accounts
            .get(&account_id)
            .unwrap_or_else(|| env::panic(b"The account is not registered"));

        if let Some(new_balance) = balance.checked_sub(amount) {
            self.accounts.insert(&account_id, &new_balance);
        } else {
            env::panic(b"The account doesn't have enough balance");
        }
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(
            sender_id != receiver_id,
            "Sender and receiver should be different"
        );

        require!(amount > 0, "The amount should be a positive number");

        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);

        FtTransferLog {
            old_owner_id: sender_id.to_string(),
            new_owner_id: receiver_id.to_string(),
            amount: U128::from(amount),
            memo,
        }
        .emit();
    }

    pub fn internal_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) -> u128 {
        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);

            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);

                self.accounts
                    .insert(&receiver_id, &(receiver_balance - refund_amount));

                if let Some(sender_balance) = self.accounts.get(&sender_id) {
                    self.accounts
                        .insert(&sender_id, &(sender_balance + refund_amount));

                    FtTransferLog {
                        old_owner_id: receiver_id.to_string(),
                        new_owner_id: sender_id.to_string(),
                        amount: U128::from(refund_amount),
                        memo: Some("refund".to_string()),
                    }
                    .emit();
                    return (amount - refund_amount).into();
                } else {
                    // Sender's account was deleted, so we need to burn tokens.
                    self.total_supply -= refund_amount;

                    log!("The account of the sender was deleted");

                    FtBurnLog {
                        owner_id: receiver_id.to_string(),
                        amount: U128::from(refund_amount),
                        memo: Some("burn".to_string()),
                    }
                    .emit();
                }
            }
        }
        amount
    }
}
