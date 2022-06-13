# Non Fungible Token

**This Contract implements smart-contract standards set by near community**

[NEP - 171](https://nomicon.io/Standards/Tokens/NonFungibleToken/Core)

`NEP - 171 is implemented for nft core functionality`

> All the basic functionalities regarding nft, their transfer, and interaction with other contracts are covered here, but here we restrict the transfer of NFT's only among Catch users (username.catchlabs.near account only)

[NEP - 178](https://nomicon.io/Standards/Tokens/NonFungibleToken/ApprovalManagement)

`NEP - 178 is implemented for approval management system`

> NEP - 178 is modified a bit in such a way that users can only approve their NFT's to Catch's Native MArketplace and other Catch Approved MArketplace, the reason for this move is to enforce royalty payouts to developers while still giving option to users to list on other marketplace which might me much better than our native marketplace and let users enjoy the latest and best of marketplaces on near

Team Catch will approve marketplaces based only 2 conditions

1. No user is being harmed or fooled and have all basic marketplace functionalities

2. Royalty payments shouldn be enforced by contract

[NEP - 181](https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration)

`NEP - 181 is implemented for enumeration purposes`

> Standard interfaces for counting & fetching tokens, for an entire NFT contract or for a given owner, also supports fetching all approved maretplaces list

[NEP - 177](https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata)

`NEP - 177 is implemented as standard for contract and token metadata`

> Standard for both Token Metadata and Contract Metadata, The Token Metadata is modified in accordance with needs for the Catch Gameplay

[NEP - 199](https://nomicon.io/Standards/Tokens/NonFungibleToken/Payout)

`NEP - 199 is implemented to provide a way for financial contracts like marketplaces to query payments and royalty information`

> An interface allowing non-fungible token contracts to request that financial contracts pay-out multiple receivers, enabling flexible royalty implementations.

### Calling the Contract from CLI

> I'm logged in as catchlabs.testnet

`export NFT_CONTRACT=nft.catchlabs.testnet`

`export OWNER=darshan3v.testnet`

#### Create Sub-Account for deploying

`near create-account $NFT_CONTRACT --masterAccount catchlabs.testnet`

#### Deploying on testnet

`near deploy $NFT_CONTRACT --wasmFile res/nft.wasm`

deployed on testnet @ `nft.catchlabs.testnet`

> Now I'm logged in as darshan3v.testnet

#### Init function

`near call $NFT_CONTRACT new '{"owner_id": "'$OWNER'","metadata": { "spec": "nft-1.0.0","name": "Catch NFT Contract","symbol": "CATCH","icon": "C-A-T-C-H","base_uri": "ipfs","reference": "ipfs://metadata/example.link","reference_hash": "AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M="}}' --accountId $OWNER`

#### nft_mint fn

> It supports Batch Minting of NFT

`export OWNER=darshan3v.catchlabs.testnet`

> Now i'm logged in as darshan3v.catchlabs.testnet

`near call $NFT_CONTRACT nft_mint '{"nft_info_list": [{"token_id": "token-1", "metadata": {"title": "Catch NFT Token", "description": "Player Darshan", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif","media_hash": "AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M="}, "receiver_id": "darshan3v.catchlabs.testnet"},{"token_id": "token-2", "metadata": {"title": "Catch NFT token", "description": "Player Andrius", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif","media_hash": "AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M="}, "receiver_id": "andrius.catchlabs.testnet"}]}' --accountId $OWNER --amount 1`

#### nft_transfer fn

> Restricted Transfers among Catch Players only

`near call $NFT_CONTRACT nft_transfer '{"receiver_id": "andrius.catchlabs.testnet","token_id": "token-1"}' --accountId $OWNER --depositYocto 1`

#### nft_transfer_call fn

`near call $NFT_CONTRACT nft_transfer_call '{"receiver_id": "some-contract.testnet", "token_id": "token-2", "msg": "foo"}' --accountId $OWNER --depositYocto 1 --gas 200000000000000`

#### nft_approve fn

`near call $NFT_CONTRACT nft_approve '{"token_id": "token-1","account_id": "marketplace.catchlabs.testnet"}' --accountId $OWNER --amount 0.1`

#### nft_is_approved fn

`near view $NFT_CONTRACT nft_is_approved '{"token_id": "token-1", "approved_account_id": "marketplace.catchlabs.testnet"}'`

#### nft_revoke fn

`near call $NFT_CONTRACT nft_revoke '{"token_id": "token-1", "account_id": "marketplace.catchlabs.testnet"}' --accountId $OWNER --depositYocto 1`

#### nft_revoke_all fn

`near call $NFT_CONTRACT nft_revoke_all '{"token_id": "token-1"}' --accountId $OWNER --depositYocto 1`

#### approve_marketplace fn

`near call $NFT_CONTRACT approve_marketplace '{"marketplace_id": "somemarketplace.testnet"}' --accountId $OWNER --amount 0.0001`

#### get_approved_marketplace fn

`near view $NFT_CONTRACT get_approved_marketplace '{"from_index": "20", "limit": 30}'`

#### nft_payout fn

`near view $NFT_CONTRACT nft_payout '{"token_id": "token-1", "balance": "30", "max_len_payout": 5}'`

#### nft_transfer_payout fn

> Called by Marketplace

#### nft_total_supply fn

`near view $NFT_CONTRACT nft_total_supply`

#### nft_tokens fn

> Note Below from_index is u128 type and hence string whereas limit is u64 so number

`near view $NFT_CONTRACT nft_tokens '{"from_index": "20", "limit": 30}'`

#### nft_supply_for_owner fn

`near view $NFT_CONTRACT nft_supply_for_owner '{"account_id": "andrius.catchlabs.tetsnet"}'`

#### nft_tokens_for_owner fn

`near view $NFT_CONTRACT nft_tokens_for_owner '{"account_id": "andrius.catchlabs.tetsnet","from_index": "20", "limit": 30}'`

#### nft_token fn

`near view $NFT_CONTRACT nft_token '{"token_id": "token-1"}'`

#### nft_metadata fn

`near view $NFT_CONTRACT nft_metadata`
