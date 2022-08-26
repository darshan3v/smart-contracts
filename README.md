# Catch smart-contracts

This Repo contains all the smart contracts related to Catch and tests related to them.

Testnet contracts going live soon !

It is basically divided into 3 contracts : 

## 1. Fungible Token
` Testnet Contract : ft.catchlabs.testnet`

```
Catch tokens are the Fungible Tokens that will be utilised for the purpose of this project

This Contract would contain all the logic related to FT & FT Reward Distribution functionality that will be in conjuction with NFT Contract

Sub-Account Creation for players will also be handled by this contract
```

## 2. Non-Fungible Token
` Testnet Contract : nft.catchlabs.testnet!`

```
This Contract would contain all the logic related to NFT & In-Game Achievements functionality that will be in conjuction with FT Contract

It would also track player achievements and their NFT's

It would also handle business side of things for them to upload and get featured in the Catch Map

These NFT's can only be listed on Catch's native Marketplace and Catch Approved marketplace to ensure developer royalties are being enforced and no harm is caused to catch users while still allowing room for competiton and innovation in marketplaces

```

## 3. Market
` Testnet Contract : marketplace.catchlabs.testnet`

```
This Contract will basically handle all the logic regarding NFT marketplace

It would handle trading of NFT's
```

All Contracts follow the Near Standards for smart-contracts with slight appropriate Modifications

You can go the appropriate Contract folders to find their appropriate Readme files.

User flow for the contract is also explained in the readme itself.

### Build Wasm 

> To build Wasm files of contracts, go the appropriate contract folder and then execute the below command

```console
./build.sh
```

