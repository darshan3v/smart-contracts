#!/usr/bin/zsh

source ~/.zshrc

# create account event_owner.test.near
local_near create-account event_owner.test.near --masterAccount test.near &&
local_near send test.near event_owner.test.near 50 &&

# create account darshan.test.near
local_near create-account darshan.test.near --masterAccount test.near &&
local_near send test.near darshan.test.near 50 &&

# create account rahul.test.near
local_near create-account rahul.test.near --masterAccount test.near &&
local_near send test.near rahul.test.near 50 &&

# create account vivek.test.near
local_near create-account vivek.test.near --masterAccount test.near &&
local_near send test.near vivek.test.near 50 