#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../ || exit 1

# Args
IC_HISTORY_ROUTER=$1
OWNER_PRINCIPAL_ID=$2

dfx canister --no-wallet \
  create nft --controller "$OWNER_PRINCIPAL_ID"

CREATED_MARKETPLACE_CANISTER_ID=$(dfx canister id marketplace)

dfx canister --no-wallet \
  update-settings \
    --controller "$OWNER_PRINCIPAL_ID" \
    --controller "$CREATED_MARKETPLACE_CANISTER_ID" \
  "$CREATED_MARKETPLACE_CANISTER_ID"

dfx deploy --no-wallet \
  marketplace --argument "(
    principal \"$IC_HISTORY_ROUTER\",
    principal \"$OWNER_ID\"
  )"
