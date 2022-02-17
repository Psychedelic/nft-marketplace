#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

DEBUG=1
DEFAULT_PRINCIPAL_ID=$(HOME=$HOME dfx identity get-principal)
IC_HISTORY_ROUTER=""

deployCapRouter() {
  printf "🤖 Deploy Cap\n"

  yarn cap:start

  IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)
}

deployDab() {
  printf "🤖 Deploy Dab\n"

  yarn dab:start
}

deployDip721() {
  printf "🤖 Deploy DIP721 NFT Canister\n"

  ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
  tokenSymbol="FOO"
  tokenName="Foobar"

  printf "🤖 Deploying NFT with %s %s %s\n" "$ownerPrincipalId" "$tokenSymbol" "$tokenName"

  yarn dip721:deploy-nft "$ownerPrincipalId" "$tokenSymbol" "$tokenName"

  nonFungibleContractAddress=$(cd ./DIP721 && dfx canister id nft)

  printf "NFT Contract address -> %s\n" "$nonFungibleContractAddress"
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  icHistoryRouter=$1
  ownerPrincipalId=$2

  yes yes | yarn marketplace:deploy "$icHistoryRouter" "$ownerPrincipalId"
}

deployWICP() {
  printf "🤖 Deploy wICP Token Canister\n"

  owner="$1"

  yarn wicp:deploy "$owner"

  printf "🤖 Balance of name (%s)" "$owner"

  yarn wicp:balance-of "$owner"

  wicpId="$(cd ./wicp && dfx canister id wicp)"

  printf "🤖 wICP Canister id is %s\n" "$wicpId"
}

deployCapRouter
[ "$DEBUG" == 1 ] && echo $?

# TODO: Check why it throws replica 404
# deployDab
# [ "$DEBUG" == 1 ] && echo $?

deployDip721
[ "$DEBUG" == 1 ] && echo $?

deployMarketplace "$IC_HISTORY_ROUTER" "$DEFAULT_PRINCIPAL_ID"
[ "$DEBUG" == 1 ] && echo $?

deployWICP "$DEFAULT_PRINCIPAL_ID"
[ "$DEBUG" == 1 ] && echo $?

echo "👍 Deploy services completed!"