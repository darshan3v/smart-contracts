#!/usr/bin/zsh

source ~/.zshrc

# create 2 events with no dependencies

# event 1
tokeninfo1='{"token_id": "bronze_medal","token_metadata": {"title":"Bronze","media":"https://images.unsplash.com/photo-1627764494888-88b31b8566df?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1025&q=80","media_hash":"AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","copies": 1000,"expires_at": 1660140358},"token_dependency_by_id": [], "event_dependency_by_id": []}' &&
tokeninfo2='{"token_id": "silver_medal","token_metadata": {"title":"Silver","media":"https://images.unsplash.com/photo-1613825787113-c9da2585b2c2?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxzZWFyY2h8MXx8c2lsdmVyJTIwbWVkYWx8ZW58MHx8MHx8&auto=format&fit=crop&w=500&q=60","media_hash":"AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","copies": 2000, "expires_at": 1660140358},"token_dependency_by_id": [], "event_dependency_by_id": []}' &&

local_near call contract.test.near organise_event '{"event_id": "CATCH-CON new","tokens":['$tokeninfo1,$tokeninfo2']}' --accountId event_owner.test.near --deposit 2 &&

# event 2
tokeninfo3='{"token_id": "trophy","token_metadata": {"title":"Trophy","media":"https://media.istockphoto.com/photos/golden-trophy-cup-on-dark-background-copy-space-for-text-3d-rendering-picture-id1296652778?b=1&k=20&m=1296652778&s=170667a&w=0&h=guJnyq9Xw1HUsdjZ7wXiKYhlaCdbRUoDxORkS_kt564=","media_hash":"AK3YRHqKhCJNmKfV6SrutnlWW/icN5J8NUPtKsNXR1M=","copies": 3},"token_dependency_by_id": [], "event_dependency_by_id": []}' &&
local_near call contract.test.near organise_event '{"event_id": "CATCH-CON2 new","tokens":['$tokeninfo3']}' --accountId event_owner.test.near --deposit 2
