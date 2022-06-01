// //! Standard for nep141 (Fungible Token) events.
// //!
// //! These events will be picked up by the NEAR indexer.
// //!
// //! <https://github.com/near/NEPs/blob/master/specs/Standards/FungibleToken/Event.md>
// //!
// //! The three events in this standard are [`FtMint`], [`FtTransfer`], and [`FtBurn`].
// //!

use std::fmt;

use crate::*;

pub const FT_STANDARD_NAME: &str = "nep141";

pub const FT_METADATA_SPEC: &str = "1.0.0";

/// Enum that represents the data type of the EventLog.
#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    FtMint(FtMintLog),
    FtTransfer(FtTransferLog),
    FtBurn(FtBurnLog),
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    // `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
    #[serde(flatten)]
    pub event: EventLogVariant,
}

impl fmt::Display for EventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtMintLog {
    pub owner_id: String,
    pub amount: U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl FtMintLog {
    pub fn emit(self) {
        let event_log = EventLog {
            standard: FT_STANDARD_NAME.to_string(),
            version: FT_METADATA_SPEC.to_string(),
            event: EventLogVariant::FtMint(self),
        };

        env::log(event_log.to_string().as_bytes());
    }
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtTransferLog {
    pub old_owner_id: String,
    pub new_owner_id: String,
    pub amount: U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl FtTransferLog {
    pub fn emit(self) {
        let event_log = EventLog {
            standard: FT_STANDARD_NAME.to_string(),
            version: FT_METADATA_SPEC.to_string(),
            event: EventLogVariant::FtTransfer(self),
        };

        env::log(event_log.to_string().as_bytes());
    }
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FtBurnLog {
    pub owner_id: AccountId,
    pub amount: U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl FtBurnLog {
    pub fn emit(self) {
        let event_log = EventLog {
            standard: FT_STANDARD_NAME.to_string(),
            version: FT_METADATA_SPEC.to_string(),
            event: EventLogVariant::FtBurn(self),
        };

        env::log(event_log.to_string().as_bytes());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod events {
    use super::*;
    use crate::utils::test_utils::*;
    use near_sdk::json_types::U128;
    use near_sdk::MockedBlockchain;
    use near_sdk::{test_utils, testing_env};

    #[test]
    fn ft_mint() {
        testing_env!(get_context(carol().to_string(), 500));
        let owner_id: AccountId = bob().into();
        let amount = U128(100);
        FtMintLog {
            owner_id: owner_id.to_string(),
            amount: amount,
            memo: None,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_mint","data":{"owner_id":"bob.near","amount":"100"}}"#
        );
    }

    #[test]
    fn ft_burn() {
        testing_env!(get_context(carol().to_string(), 500));
        let owner_id: AccountId = bob().into();
        let amount = U128(100);
        FtBurnLog {
            owner_id: owner_id.to_string(),
            amount: amount,
            memo: None,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_burn","data":{"owner_id":"bob.near","amount":"100"}}"#
        );
    }

    #[test]
    fn ft_transfer() {
        testing_env!(get_context(carol().to_string(), 500));
        let old_owner_id: AccountId = bob().into();
        let new_owner_id: AccountId = alice().into();
        let amount = U128(100);
        FtTransferLog {
            old_owner_id: old_owner_id.to_string(),
            new_owner_id: new_owner_id.to_string(),
            amount: amount,
            memo: None,
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_transfer","data":{"old_owner_id":"bob.near","new_owner_id":"alice.near","amount":"100"}}"#
        );
    }
}
