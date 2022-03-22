#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../ || exit 1

# Args
_owner=$1
_icHistoryRouter=$2

dfx canister \
  create marketplace --controller "$_owner"

_createdMarketCanisterId=$(dfx canister id marketplace)

dfx canister \
  update-settings \
    --controller "$_owner" \
    --controller "$_createdMarketCanisterId" \
  "$_createdMarketCanisterId"

dfx deploy \
  marketplace --argument "(
    principal \"$_icHistoryRouter\",
    principal \"$_owner\"
  )"
