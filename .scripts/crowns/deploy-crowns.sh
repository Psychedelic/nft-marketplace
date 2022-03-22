#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../crowns || exit 1

_owner=$1
_tokenName=$2
_tokenSymbol=$3

dfx canister \
  create crowns \
  --controller "$_owner"

dfx build crowns

_nonFungibleContractAddress=$(dfx canister id crowns)

dfx canister \
  update-settings \
    --controller "$_owner" \
    --controller "$_nonFungibleContractAddress" \
  "$_nonFungibleContractAddress"

dfx deploy \
  crowns --argument "(
    opt record {
      name = opt \"$_tokenName\";
      logo = opt \"data:image/jpeg;base64,...\";
      symbol = opt \"$_tokenSymbol\";
      owners = opt vec { principal \"$_owner\" };
    }
)"