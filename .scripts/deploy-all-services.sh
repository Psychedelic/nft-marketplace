#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)

DEFAULT_USER_WALLET=$(dfx identity get-wallet)

deployCapRouter() {
  printf "🤖 Deploy Cap\n"

  yarn cap:start

  _icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)

  printf "CAP IC History router -> %s\n" "$_icHistoryRouter"

  IC_HISTORY_ROUTER=$_icHistoryRouter
}

deployDab() {
  printf "🤖 Deploy Dab\n"

  yarn dab:start
}

deployDip721() {
  printf "🤖 Deploy DIP721 Crowns NFT Canister\n"

  _owner_wallet=$1
  _tokenName=$2
  _tokenSymbol=$3

  yarn crowns:deploy "$_owner_wallet" "$_tokenName" "$_tokenSymbol"
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  _ownerPrincipalId=$1
  _icHistoryRouter=$2

  yes yes | yarn marketplace:deploy "$_icHistoryRouter" "$_ownerPrincipalId"
}

deployWICP() {
  printf "🤖 Deploy wICP Token Canister\n"

  _wallet="$1"
  _ic_history_router="$2"
  _amount="$3"

  yarn wicp:deploy "$_wallet" "$_ic_history_router" "$_amount"

  printf "🤖 Balance of name (%s)" "$_wallet"

  yarn wicp:balance-of "$_wallet"

  _wicpId="$(cd ./wicp && dfx canister id wicp)"

  printf "🤖 wICP Canister id is %s\n" "$_wicpId"
}

deployCapRouter
[ "$DEBUG" == 1 ] && echo $?

# TODO: Check why it throws replica 404
# deployDab
# [ "$DEBUG" == 1 ] && echo $?

deployDip721 "$DEFAULT_USER_WALLET" "Crowns" "CRW"
[ "$DEBUG" == 1 ] && echo $?

deployMarketplace "$DEFAULT_USER_WALLET" "$IC_HISTORY_ROUTER" 
[ "$DEBUG" == 1 ] && echo $?

deployWICP "$DEFAULT_USER_WALLET" "$IC_HISTORY_ROUTER" 1_000_000_000_000""
[ "$DEBUG" == 1 ] && echo $?

echo "👍 Deploy services completed!"