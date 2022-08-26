use crate::*;

/// external contract calls

//initiate a cross contract call to the nft contract.This will transfer the token to the buyer and return
//a payout object used for the market to distribute funds to the appropriate accounts.
#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId, //purchaser (person to transfer the NFT to)
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
        max_len_payout: u32,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}
