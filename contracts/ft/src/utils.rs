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

pub(crate) fn is_valid_prefix(prefix: &near_sdk::AccountId) -> bool {
    for c in prefix.as_bytes() {
        match c {
            b'-' | b'_' | b'.' => return false,
            _ => (),
        }
    }

    // ToDo -> Also check if the prefix is not a reserved keyword like users,settings,dao.....
    return true;
}
