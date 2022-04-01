use crate::core_impl::FungibleToken;

use near_sdk::json_types::U128;
use near_sdk::{assert_self, ext_contract, AccountId, Balance};

#[ext_contract(ext_self)]
pub trait FungibleTokenResolverExt {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

pub trait FungibleTokenResolver {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

impl FungibleTokenResolver for FungibleToken {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        assert_self(); // Private Function

        let amount: Balance = amount.into();

        self.internal_resolve_transfer(&sender_id, &receiver_id, amount)
            .into()
    }
}
