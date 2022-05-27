#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../crowns || exit 1

_owner=$1
_testUser1=$2
_testUser2=$3
_tokenName=$4
_tokenSymbol=$5
_cap=$6

# On the mock system for Crowns
# a system principal id is used on `mint`
# find in `crowns/mocks/principals.js`
crownsMockSystemPrincipalId="a2t6b-nznbt-igjd3-ut25i-b43cf-mt45v-g3x2g-ro6h5-kowno-dx3rz-uqe"

dfx canister \
  create crowns \
  --controller "$_owner"

dfx build crowns

_nonFungibleContractAddress=$(dfx canister id crowns)

dfx canister \
  update-settings \
    --add-controller "$_owner" \
    --add-controller "$_nonFungibleContractAddress" \
  "$_nonFungibleContractAddress"

dfx deploy \
  crowns --argument "(
    opt record {
      name = opt \"$_tokenName\";
      logo = opt \"data:image/jpeg;base64,...\";
      symbol = opt \"$_tokenSymbol\";
      owners = opt vec { principal \"$_owner\" };
      cap = opt principal \"$_cap\";
    }
)"

dfx canister \
  call --update "$_nonFungibleContractAddress" \
  setCustodians "( 
    vec {
      principal \"$crownsMockSystemPrincipalId\";
      principal \"$_owner\";
      principal \"$testUser1\";
      principal \"$testUser2\";
    } 
  )"