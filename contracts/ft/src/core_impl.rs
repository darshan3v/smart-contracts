use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 5_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

const NO_DEPOSIT: Balance = 0;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, Balance>,

    /// Total supply of the FT token.
    pub total_supply: Balance,
}

pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: Balance, memo: Option<String>);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: Balance,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_total_supply(&self) -> U128;

    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

/****************************************************/
/*  IMPLEMENTING FUNGIBLE TOKEN CORE FUNCTIONALITY  */
/****************************************************/

impl FungibleTokenCore for FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: Balance, memo: Option<String>) {
        assert_one_yocto();

        let sender_id = env::predecessor_account_id();

        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: Balance,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        require!(
            env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL + GAS_FOR_RESOLVE_TRANSFER,
            "More gas is required"
        );

        let sender_id = env::predecessor_account_id();

        self.internal_transfer(&sender_id, &receiver_id, amount, memo);

        // Initiating receiver's call and the callback

        ext_fungible_token_receiver::ft_on_transfer(
            sender_id.clone(),
            amount.into(),
            msg,
            &receiver_id,
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::ft_resolve_transfer(
            sender_id,
            receiver_id,
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.accounts.get(&account_id).unwrap_or(0).into()
    }
}
