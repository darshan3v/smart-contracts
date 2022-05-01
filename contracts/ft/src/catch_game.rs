use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::Serialize;
use near_sdk::{near_bindgen, Balance};
use std::convert::From;

/************/
/*   Note   */
/************/

// obj_metadata and stats is in the same order as objectives

// Both of them should be validated before deploying the contract itself

/******************/
/*   Assumption   */
/******************/

// Number of Achievements will be <= 2^8

/// Denotes Rarity of a Objective according to how many of them are able to achieve it
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Rarity {
    Common,
    Rare,
    Legendary,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CatchObjectives {
    /// BaseData for InGame Objectives
    pub objectives: Vector<Objective>,

    /// Metadata for InGame Objectives
    pub obj_metadata: LazyOption<Vector<ObjectiveMetadata>>,

    /// Stats Regarding InGame Objectives
    pub stats: Vector<ObjectiveStats>,
}

// Read Only

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Objective {
    /// Title name of the Objective
    pub title: String,

    /// Amount of Catch Tokens to be awarded on completion of Objective
    pub reward: Balance,
}

// Read Only

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ObjectiveMetadata {
    /// An IPFS link to JSON file containing detalis of the achievement [link,description,etc....]
    pub reference: String,

    /// SHA-256 hash of JSON file pointed by above url to prevent Off - Chain Tampering
    pub reference_hash: Base64VecU8,
}

// Read and Write Both Allowed

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ObjectiveStats {
    /// Number of people who have accomplished the Objective
    pub winner_count: u128,

    /// Rarity : That is is the objective very rare ?
    pub rarity: Rarity,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonObjective {
    pub objective: Objective,

    pub obj_metadata: ObjectiveMetadata,

    pub stats: ObjectiveStats,
}

impl CatchObjectives {
    pub fn default() -> CatchObjectives {
        let mut objectives = Vector::new(StorageKey::Objective.try_to_vec().unwrap());
        objectives.extend(Objective::default_list());

        let mut obj_metadata_list =
            Vector::new(StorageKey::ObjectiveMetadataList.try_to_vec().unwrap());
        obj_metadata_list.extend(ObjectiveMetadata::default_list());

        let obj_metadata = LazyOption::new(
            StorageKey::ObjectiveMetadata.try_to_vec().unwrap(),
            Some(&obj_metadata_list),
        );

        let mut stats = Vector::new(StorageKey::ObjectiveStats.try_to_vec().unwrap());
        stats.extend(ObjectiveStats::default_list());

        CatchObjectives {
            objectives,
            obj_metadata,
            stats,
        }
    }

    // It will increase the winner_count by 1 , and update rarity
    pub fn update_stats(&mut self, index: u8) {
        // It is safe to query stats with the index given, bcz it is only called by transfer_player_reward()
        let obj_stat = self
            .stats
            .get(index.into())
            .unwrap_or_else(|| env::panic(b"Invariant : Invalid Objective"));

        let new_winner_count = obj_stat.winner_count + 1;

        let new_rarity = match new_winner_count {
            0..=1000 => Rarity::Legendary,
            1001..=20000 => Rarity::Rare,
            _ => Rarity::Common,
        };

        let new_stat = ObjectiveStats {
            winner_count: new_winner_count,
            rarity: new_rarity,
        };

        self.stats.replace(index.into(), &new_stat);
    }
}

impl Objective {
    pub fn default_list() -> Vec<Objective> {
        vec![
            Objective {
                title: String::from("Objective1"),
                reward: 1000,
            },
            Objective {
                title: String::from("Objective2"),
                reward: 2000,
            },
        ]
    }
}

impl ObjectiveMetadata {
    pub fn default_list() -> Vec<ObjectiveMetadata> {
        vec![
            ObjectiveMetadata {
                reference: "ipfs://achievement1".to_string(),
                reference_hash: Base64VecU8::from([1_u8; 32].to_vec()),
            },
            ObjectiveMetadata {
                reference: "ipfs://achievement2".to_string(),
                reference_hash: Base64VecU8::from([2_u8; 32].to_vec()),
            },
        ]
    }
}

impl ObjectiveStats {
    pub fn default_list() -> Vec<ObjectiveStats> {
        vec![
            ObjectiveStats {
                winner_count: 0,
                rarity: Rarity::Common,
            },
            ObjectiveStats {
                winner_count: 0,
                rarity: Rarity::Common,
            },
        ]
    }
}

impl From<(Objective, ObjectiveMetadata, ObjectiveStats)> for JsonObjective {
    fn from(item: (Objective, ObjectiveMetadata, ObjectiveStats)) -> Self {
        Self {
            objective: item.0,
            obj_metadata: item.1,
            stats: item.2,
        }
    }
}

// ToDo

// Ensure that for every ping on NFT contract some 5 to 7 (yet to calculate) index can only be sent because of the gas limit , we cannot perform unlimited operation

// Ensure that NFT Contract callback function checks if transfer_player_reward() ran successfully then make state changes like registering those achievements as rewarded it should not be done before callback

#[near_bindgen]
impl Contract {
    /// Transfer Fungible Token Rewards to players
    pub fn transfer_player_reward(&mut self, player_id: AccountId, obj_index: Base64VecU8) {
        require!(
            env::predecessor_account_id().as_str() == "nft.catchlabs.near",
            "Reward distribution can only be handled by CatchLabs NFT Contract"
        );

        require!(
            self.token.accounts.contains_key(&player_id),
            "Player is Not Registered with the FT contract"
        );

        let mut prize: Balance = 0;
        let indexes = obj_index.0;
        let mut objective;

        let owner_id = self.owner_id.clone();

        for i in indexes.into_iter() {
            objective = self
                .catch_objectives
                .objectives
                .get(i.into())
                .unwrap_or_else(|| env::panic(b"Invariant : Invalid Objective"));

            prize += objective.reward;
            self.catch_objectives.update_stats(i);
        }

        self.token.internal_withdraw(&owner_id, prize);
        self.token.internal_deposit(&player_id, prize);

        // ToDo - Transfer Reward Event
    }

    /// View Function - returns paginated view of Objectives Info
    pub fn get_objectives(&self, from_index: u8, limit: u8) -> Vec<JsonObjective> {
        self.catch_objectives
            .objectives
            .iter()
            .zip(self.catch_objectives.obj_metadata.get().unwrap().iter())
            .zip(self.catch_objectives.stats.iter())
            .skip(from_index.into())
            .take(limit.into())
            .map(|((obj, obj_meta), stat)| JsonObjective::from((obj, obj_meta, stat)))
            .collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod fungible_token_tests {
    use super::*;
    use near_sdk::json_types::Base64VecU8;
    use near_sdk::Balance;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const ONE_YOCTO: Balance = 1;

    // Helper functions
    fn carol() -> ValidAccountId {
        ValidAccountId::try_from("carol.near").unwrap()
    }
    fn dex() -> ValidAccountId {
        ValidAccountId::try_from("dex.near").unwrap()
    }
    fn nft() -> ValidAccountId {
        ValidAccountId::try_from("nft.catchlabs.near").unwrap()
    }

    fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: "mike.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    fn create_contract() -> Contract {
        let metadata = FungibleTokenMetadata {
            spec: String::from("1.1.0"),
            name: String::from("CAT Token"),
            symbol: String::from("CAT"),
            icon: Some(String::from("C-A-T-C-H")),
            reference: String::from(
                "https://github.com/near/core-contracts/tree/master/w-near-141",
            ),
            reference_hash: Base64VecU8::from([5_u8; 32].to_vec()),
            decimals: 0,
        };
        let total_supply = U128::from(1_000_000_000_000_000);
        Contract::new(dex(), total_supply, metadata)
    }

    #[test]
    #[should_panic(expected = "Reward distribution can only be handled by CatchLabs NFT Contract")]
    fn transfer_reward_invalid_caller() {
        testing_env!(get_context(dex().to_string(), 0));

        let mut contract = create_contract();
        let player = carol().to_string();
        let indexes = Base64VecU8::from([0].to_vec());

        contract.transfer_player_reward(player, indexes);
    }

    #[test]
    #[should_panic(expected = "Player is Not Registered with the FT contract")]
    fn transfer_reward_invalid_player() {
        testing_env!(get_context(nft().to_string(), 0));

        let mut contract = create_contract();
        let player = carol().to_string();
        let indexes = Base64VecU8::from([0].to_vec());

        contract.transfer_player_reward(player, indexes);
    }

    #[test]
    #[should_panic(expected = "Invalid Objective")]
    fn transfer_reward_invalid_objective() {
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));

        let mut contract = create_contract();
        contract.storage_deposit(Some(carol()));

        testing_env!(get_context(nft().to_string(), 0));

        let player = carol().to_string();
        let indexes = Base64VecU8::from([2].to_vec());

        contract.transfer_player_reward(player, indexes);
    }

    #[test]
    fn transfer_reward() {
        testing_env!(get_context(dex().to_string(), ONE_YOCTO));

        let mut contract = create_contract();
        contract.storage_deposit(Some(carol()));

        testing_env!(get_context(nft().to_string(), 0));

        let player = carol().to_string();
        let indexes = Base64VecU8::from([0, 1].to_vec());

        let rewards = 3000;

        contract.transfer_player_reward(player, indexes);
        assert_eq!(contract.ft_balance_of(carol()).0, rewards);
    }

    #[test]
    fn get_objectives() {
        testing_env!(get_context(dex().to_string(), 0));

        let contract = create_contract();

        let json_objs = contract.get_objectives(2, 3);

        let expected = vec![
            JsonObjective {
                objective: Objective {
                    title: String::from("Objective1"),
                    reward: 1000,
                },
                obj_metadata: ObjectiveMetadata {
                    reference: "ipfs://achievement1".to_string(),
                    reference_hash: Base64VecU8::from([1_u8; 32].to_vec()),
                },
                stats: ObjectiveStats {
                    winner_count: 0,
                    rarity: Rarity::Common,
                },
            },
            JsonObjective {
                objective: Objective {
                    title: String::from("Objective2"),
                    reward: 2000,
                },
                obj_metadata: ObjectiveMetadata {
                    reference: "ipfs://achievement2".to_string(),
                    reference_hash: Base64VecU8::from([2_u8; 32].to_vec()),
                },
                stats: ObjectiveStats {
                    winner_count: 0,
                    rarity: Rarity::Common,
                },
            },
        ];

        todo!()
        // ToDo assert expected and returned value are same
    }
}
