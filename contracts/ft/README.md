# Fungible Token 

**This Contract implements smart-contract standards set by near community**

[NEP - 141](https://nomicon.io/Standards/Tokens/FungibleToken/Core)

`NEP - 141 is implemented for ft core functionality`

> All the basic functionalities regarding ft, their transfer, and interaction with other contracts are covered here

[NEP - 145](https://nomicon.io/Standards/StorageManagement)

`NEP - 145 is implemented for storage functionality`

> There is no storage withdraw function implemented because in this scenario it's the same as storage_unregister


> There is no need of explicit near deposit for players if catchlabs is calling storage_deposit function as the contract (aka catchlabs) will fund these users storage

[NEP - 148](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata)

`NEP - 148 is implemented for FT metadata standards`

### Reward Distribution For Catch Players

#### Assumption

> There are less than 128 In-Game Objectives ( For now, if required can be changed in contract )

#### Explanation

> NFT contract for catch will check if a player has completed any in game objective, and if yes then it will issue a call to this ft contract to distribute rewards to these players

> Objectives related most of the data will be hardcoded as it is much cheaper to hardcode it before deploying, it is the deployers responsibility to validate if all the Hardcoded Objectives are valid, consistent and correct

> Functionality to add Objectives using function call can be added easily and will be added if required

> For now FT contract has the title, reward amount, obj_metadata ipfs links, and obj_stats such as how many players have completed this objective, and accordingly updates if the objective is legendary type or so ...

### Calling the Contract from CLI

> I'm logged in as darshan3v.testnet 

`export FT_CONTRACT=ft.darshan3v.testnet`

`export OWNER=darshan3v.testnet`

#### Create Sub-Account for deploying 

`near create-account $FT_CONTRACT --masterAccount $OWNER`

#### Deploying on testnet

`near deploy ft.darshan3v.testnet --wasmFile target/wasm32-unknown-unknown/release/ft.wasm`

deployed on testnet @ `ft.darshan3v.testnet`

#### Init function

`near call $FT_CONTRACT new '{"owner_id": "'$OWNER'","total_supply": "1000000000","metadata": { "spec": "ft v1.0.0","name": "CAT Token","symbol": "CATCH","icon": "C-A-T-C-H","reference": "ipfs://metadata/example.link","reference_hash": "AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","decimals": 0}}' --accountId $OWNER`

#### storage_deposit fn

`near call $FT_CONTRACT storage_deposit '{"account_id": "andrius.testnet"}' --accountId $OWNER --depositYocto 1`

#### storage_unregister fn

`near call $FT_CONTRACT storage_unregister '{"force": false}' --accountId $OWNER --depositYocto 1`

#### storage_balance_bounds fn

`near view $FT_CONTRACT storage_balance_bounds`

#### storage_balance_of fn

`near view $FT_CONTRACT storage_balance_of '{"account_id": "'$OWNER'"}'`

#### ft_transfer fn

`near call $FT_CONTRACT ft_transfer '{"receiver_id": "andrius.testnet","amount": "100000", "memo": "testing ft_transfer" }' --accountId $OWNER --depositYocto 1`

#### ft_transfer_call fn

`near call $FT_CONTRACT ft_transfer_call '{"receiver_id": "some_contract.testnet","amount": "100000", "memo": "testing ft_transfer_call","msg": "args to pass to called contract" }' --accountId $OWNER --depositYocto 1`

#### ft_balance_of fn

`near view $FT_CONTRACT ft_balance_of '{"account_id": "andrius.testnet"}'`

#### ft_total_supply

`near view $FT_CONTRACT ft_total_supply`

#### ft_metadata

`near view $FT_CONTRACT ft_metadata`
