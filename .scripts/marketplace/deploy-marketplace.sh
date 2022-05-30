#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../ || exit 1

# Args
_owner=$1
_icHistoryRouter=$2
_protocol_fee=$3

dfx canister \
  create marketplace --controller "$_owner"

_createdMarketCanisterId=$(dfx canister id marketplace)

dfx canister \
  update-settings \
  "$_createdMarketCanisterId" \
  --add-controller "$_createdMarketCanisterId" 


dfx deploy \
  marketplace --argument "(
    principal \"$_owner\",
    $_protocol_fee:nat,
    principal \"$_icHistoryRouter\",
  )" \
  $4
