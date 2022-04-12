use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, Balance};

// Can be accessed using index

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Achievement {
    /// A small title is preferred
    pub title: String,

    /// Amount of Catch Tokens to be rewarded
    pub reward: u128,

    /// Number of players who have won this achievement
    pub winner_count: u128,
}

// Can be accessed using index, Metadata is in the same order as Achievement Vector

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AchievementMetadata {
    /// An IPFS link to JSON file containing detalis of the achievement [link,description,etc....]
    pub reference: String,

    /// SHA-256 of JSON file pointed by above url to prevent Off - Chain Tampering
    pub reference_hash: Base64VecU8,
}

impl Achievement {
    pub fn new() -> Vec<Achievement> {
        let achievement_list = vec![
            Achievement {
                title: "Achievement - 1".to_string(),
                reward: 5000,
                winner_count: 0,
            },
            Achievement {
                title: "Achievement - 2".to_string(),
                reward: 10000,
                winner_count: 0,
            },
        ];

        achievement_list
    }
}

impl AchievementMetadata {
    pub fn new() -> Vec<AchievementMetadata> {
        let achievement_metadata_list = vec![
            AchievementMetadata {
                reference: "ipfs://achievement1".to_string(),
                reference_hash: Base64VecU8::from([1_u8; 32].to_vec()),
            },
            AchievementMetadata {
                reference: "ipfs://achievement1".to_string(),
                reference_hash: Base64VecU8::from([2_u8; 32].to_vec()),
            },
        ];

        achievement_metadata_list
    }
}

// ToDo

// Assumption : number of Achievements will be <= 2^8

// implement a paginated view of all achievements metadata

// Ensure that for every ping on NFT contract some 5 to 7 (yet to calculate) index can only be sent because of the gas limit , we cannot perform unlimited operation

// Ensure that NFT Contract callback function checks if transfer_player_reward() ran successfully then make state changes like registering those achievements as rewarded it should not be done before callback

#[near_bindgen]
impl Contract {
    /// Transfer Fungible Token Rewards to players
    pub fn transfer_player_reward(&mut self, player_id: AccountId, achievement_index: Base64VecU8) {
        require!(
            env::predecessor_account_id().as_str() == "nft.catchlabs.near",
            "Reward distribution can only be handled by CatchLabs NFT Contract"
        );

        require!(
            self.token.accounts.contains_key(&player_id),
            "Player is Not Registered with the FT contract"
        );

        let mut amount: Balance = 0;
        let mut achievement: Achievement;
        let indexes = achievement_index.0;

        let owner_id = self.owner_id.clone();
        let player_id: AccountId = player_id.into();

        for i in indexes.into_iter() {
            achievement = self
                .achievements
                .get(i.into())
                .unwrap_or_else(|| env::panic(b"Invalid Achievement index"));

            amount += achievement.reward;

            achievement.winner_count += 1;

            self.achievements.replace(i.into(), &achievement);
        }

        self.token.internal_withdraw(&owner_id, amount);
        self.token.internal_deposit(&player_id, amount);

        // ToDo - Transfer Reward Event
    }
}
