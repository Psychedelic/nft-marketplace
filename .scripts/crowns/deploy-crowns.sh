#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../crowns || exit 1

_owner_wallet=$1
_tokenName=$2
_tokenSymbol=$3

dfx canister --wallet "$_owner_wallet" \
  create crowns \
  --controller "$_owner_wallet"

dfx build crowns

nonFungibleContractAddress=$(dfx canister id crowns)

dfx canister --wallet "$_owner_wallet" \
  update-settings \
    --controller "$_owner_wallet" \
    --controller "$nonFungibleContractAddress" \
  "$nonFungibleContractAddress"

dfx deploy --wallet "$_owner_wallet" \
  crowns --argument "(
    opt record {
      name = opt \"$_tokenName\";
      logo = opt \"data:image/jpeg;base64,...\";
      symbol = opt \"$_tokenSymbol\";
      owners = opt vec { principal \"$_owner_wallet\" };
    }
)"