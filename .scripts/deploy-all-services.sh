#!/bin/bash

DEFAULT_PRINCIPAL_ID=$(dfx identity get-principal)
IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)

printf "🤖 Deploy Cap\n"

yarn cap:start

printf "🤖 Deploy wICP with owner %s\n" "$DEFAULT_PRINCIPAL_ID"

yarn wicp:deploy "$DEFAULT_PRINCIPAL_ID"

yes yes | yarn marketplace:deploy "$IC_HISTORY_ROUTER" "$DEFAULT_PRINCIPAL_ID"

printf "🤖 Deploy Dab\n"

yarn dab:start
