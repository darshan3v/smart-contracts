#!/usr/bin/zsh

source ~/.zshrc

# create account for contract and then send 50 near to contract

local_near create-account contract.test.near --masterAccount test.near && local_near send test.near contract.test.near 50 &&

# now deploy the contract

local_near deploy --accountId contract.test.near --wasmFile ./res/nft.wasm &&

# initializing 
local_near call contract.test.near new_default_meta '{"owner_id":"contract.test.near"}' --accountId contract.test.near