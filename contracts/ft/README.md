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
