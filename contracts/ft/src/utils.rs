use crate::Contract;

#[macro_export]
macro_rules! require {
    ( $a:expr, $b:expr ) => {
        if !$a {
            near_sdk::env::panic($b.as_bytes());
        }
    };
}

impl Contract {
    /// Assert that Predecessor A/c is the Owner of the Contract
    pub fn assert_owner(&self) {
        require!(
            near_sdk::env::predecessor_account_id() == self.owner_id,
            "It is a owner only method"
        );
    }
}
