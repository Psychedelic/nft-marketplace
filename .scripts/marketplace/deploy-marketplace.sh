#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../ || exit 1

# Args
_wallet=$1
_icHistoryRouter=$2

dfx canister --wallet "$_wallet" \
  create marketplace --controller "$_wallet"

_createdMarketCanisterId=$(dfx canister id marketplace)

dfx canister --wallet "$_wallet" \
  update-settings \
    --controller "$_wallet" \
    --controller "$_createdMarketCanisterId" \
  "$_createdMarketCanisterId"

dfx deploy --wallet "$_wallet" \
  marketplace --argument "(
    principal \"$_icHistoryRouter\",
    principal \"$_wallet\"
  )"
