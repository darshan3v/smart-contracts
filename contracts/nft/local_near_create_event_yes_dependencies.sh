#!/usr/bin/zsh

source ~/.zshrc

# create event with dependencies

# event dependency -> CATCH-CON, token dependency -> CATCH-CON2_Trophy

tokeninfo4='{"metadata": {"title":"Winner","media":"https://images.unsplash.com/photo-1634454686481-dff1eaa44c21?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxzZWFyY2h8OXx8d2lufGVufDB8fDB8fA%3D%3D&auto=format&fit=crop&w=500&q=60","media_hash":"AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","copies": 3},"dependencies_by_token_id":["CATCH-CON2_Trophy"],"dependencies_by_event_id":["CATCH-CON"]}' &&
local_near call contract.test.near create_event '{"event_id": "CATHC-CON-FINAL","event_tokens":['$tokeninfo4']}' --accountId event_owner.test.near --deposit 2