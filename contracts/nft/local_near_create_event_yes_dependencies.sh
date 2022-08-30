#!/usr/bin/zsh

source ~/.zshrc

# create event with dependencies

# event dependency -> CATCH-CON, token dependency -> CATCH-CON2_Trophy

tokeninfo4='{"token_id": "winner","token_metadata": {"title":"Winner","media":"https://images.unsplash.com/photo-1634454686481-dff1eaa44c21?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxzZWFyY2h8OXx8d2lufGVufDB8fDB8fA%3D%3D&auto=format&fit=crop&w=500&q=60","media_hash":"AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","copies": 3000},"token_dependency_by_id":["CATCH-CON2 new.trophy"],"event_dependency_by_id":[]}' &&
local_near call contract.test.near organise_event '{"event_id": "CATCH-CON-FINAL new","tokens":['$tokeninfo4']}' --accountId event_owner.test.near --deposit 2