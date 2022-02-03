#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../DIP721 || exit 1

CAP_IC_HISTORY_ROUTER_ID=$(cd ../../cap && dfx canister id ic-history-router)

# Args
OWNER_PRINCIPAL_ID=$1
DIP721_TOKEN_SHORT=$2
DIP721_TOKEN_NAME=$3

dfx deploy --no-wallet nft --argument "(
  principal \"$OWNER_PRINCIPAL_ID\",
  \"$DIP721_TOKEN_SHORT\",
  \"$DIP721_TOKEN_NAME\",
  principal \"$CAP_IC_HISTORY_ROUTER_ID\"
)"
