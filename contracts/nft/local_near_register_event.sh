#!/usr/bin/zsh

source ~/.zshrc

# darshan minting Bronze NFT which has no dependencies

local_near call contract.test.near event_register '{"token_id":"CATCH-CON new.bronze_medal"}' --accountId darshan.test.near --deposit 1

# rahul minting Silver NFT which has no dependencies

local_near call contract.test.near event_register '{"token_id":"CATCH-CON new.silver_medal"}' --accountId rahul.test.near --deposit 1

# vivek minting Bronze and Trophy NFT which has no dependencies and Winner which has CATCH-CON as event dependency and CATCH-CON2.trophy as token dependency

local_near call contract.test.near event_register '{"token_id":"CATCH-CON new.bronze_medal"}' --accountId vivek.test.near --deposit 1
local_near call contract.test.near event_register '{"token_id":"CATCH-CON2 new.trophy"}' --accountId vivek.test.near --deposit 1

local_near call contract.test.near event_register '{"token_id":"CATCH-CON-FINAL new.winner"}' --accountId vivek.test.near --deposit 1