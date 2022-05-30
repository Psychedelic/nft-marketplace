#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)

DEFAULT_PRINCIPAL_ID=$(dfx identity get-principal)

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

  _owner=$1
  _tokenName=$2
  _tokenSymbol=$3
  _icHistoryRouter=$4

  yarn crowns:deploy "$_owner" "$_tokenName" "$_tokenSymbol" "$_icHistoryRouter"
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  _owner=$1
  _icHistoryRouter=$2

  yes yes | yarn marketplace:deploy "$_owner" "$_icHistoryRouter" 250
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

  # set allowance to system mock generator id
  # a2t6b-nznbt-igjd3-ut25i-b43cf-mt45v-g3x2g-ro6h5-kowno-dx3rz-uqe
  _mockSystemIdentity="a2t6b-nznbt-igjd3-ut25i-b43cf-mt45v-g3x2g-ro6h5-kowno-dx3rz-uqe"

  printf "🤖 wICP topup for mock system identity %s\n" "$_mockSystemIdentity"

  dfx canister \
    call --update "$_wicpId" \
    transfer "( 
      principal \"$_mockSystemIdentity\",
      9_000_000_000_000:nat,
    )"  
}

deployCapRouter
[ "$DEBUG" == 1 ] && echo $?

# TODO: Check why it throws replica 404
# deployDab
# [ "$DEBUG" == 1 ] && echo $?

deployDip721 "$DEFAULT_PRINCIPAL_ID" "$ALICE_PRINCIPAL_ID" "$BOB_PRINCIPAL_ID" "Crowns" "CRW" "$IC_HISTORY_ROUTER" 
[ "$DEBUG" == 1 ] && echo $?

deployMarketplace "$DEFAULT_PRINCIPAL_ID" "$IC_HISTORY_ROUTER" 
[ "$DEBUG" == 1 ] && echo $?

deployWICP "$DEFAULT_PRINCIPAL_ID" "$IC_HISTORY_ROUTER" 1_000_000_000_000_000""
[ "$DEBUG" == 1 ] && echo $?

echo "👍 Deploy services completed!"