#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../DIP721 || exit 1

CAP_IC_HISTORY_ROUTER_ID=$(cd ../../cap && dfx canister id ic-history-router)

DIP721_TOKEN_SHORT=CRW
DIP721_TOKEN_NAME=Crown

dfx deploy --no-wallet nft --argument "(principal \"$(dfx identity get-principal)\", \"$DIP721_TOKEN_SHORT\", \"$DIP721_TOKEN_NAME\", principal \"$CAP_IC_HISTORY_ROUTER_ID\")"