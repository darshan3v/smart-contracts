#!/usr/bin/zsh

source ~/.zshrc

# darshan minting Bronze NFT which has no dependencies

# local_near call contract.test.near event_register '{"token_id":"CATCH-CON_Bronze"}' --accountId darshan.test.near --deposit 1

# # rahul minting Silver NFT which has no dependencies

# local_near call contract.test.near event_register '{"token_id":"CATCH-CON_Silver"}' --accountId rahul.test.near --deposit 1

# # vivek minting Bronze and Trophy NFT which has no dependencies and Winner which has CATCH-CON as event dependency and CATCH-CON2_Trophy as token dependency

# local_near call contract.test.near event_register '{"token_id":"CATCH-CON_Bronze"}' --accountId vivek.test.near --deposit 1
# local_near call contract.test.near event_register '{"token_id":"CATCH-CON2_Trophy"}' --accountId vivek.test.near --deposit 1

local_near call contract.test.near event_register '{"token_id":"CATHC-CON-FINAL_Winner"}' --accountId vivek.test.near --deposit 1