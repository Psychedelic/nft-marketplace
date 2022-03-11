#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

DEBUG=1
DEFAULT_PRINCIPAL_ID=$(dfx identity get-principal)
IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)

DEFAULT_USER_WALLET=$(dfx identity get-wallet)

deployCapRouter() {
  printf "🤖 Deploy Cap\n"

  yarn cap:start

  IC_HISTORY_ROUTER=$(cd ./cap && dfx canister id ic-history-router)

  printf "CAP IC History router -> %s\n" "$IC_HISTORY_ROUTER"
}

deployDab() {
  printf "🤖 Deploy Dab\n"

  yarn dab:start
}

deployDip721() {
  printf "🤖 Deploy DIP721 Crowns NFT Canister\n"

  (
    cd ./crowns || exit 1

    _owner_wallet=$1
    _tokenName=$2
    _tokenSymbol=$3

    printf "🤖 Deploying NFT with owner id (%s), token (%s), token name (%s), cap (%s)\n" "$ownerPrincipalId" "$tokenSymbol" "$tokenName" "$IC_HISTORY_ROUTER"

    # TODO: Refactor the dip721:deploy-nft or remove it
    # yarn dip721:deploy-nft "local" "$ownerPrincipalId" "$tokenSymbol" "$tokenName" "$IC_HISTORY_ROUTER"

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
  )
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  icHistoryRouter=$1
  ownerPrincipalId=$2

  yes yes | yarn marketplace:deploy "$icHistoryRouter" "$ownerPrincipalId"
}

deployWICP() {
  printf "🤖 Deploy wICP Token Canister\n"

  _wallet="$1"
  _ic_history_router="$2"

  # TODO: refactor the wicp:deploy
  # yarn wicp:deploy "$owner"

  (
    cd ./wicp || exit 1

    dfx deploy \
    wicp --argument="(
            \"data:image/jpeg;base64,$(base64 ../.repo/images/logo-of-wicp.png)\",
            \"wicp\",
            \"WICP\",
            8:nat8,
            1_000_000_000_000:nat,
            principal \"$_wallet\", 
            0, 
            principal \"$_wallet\", 
            principal \"$_ic_history_router\"
            )" 
  )

  printf "🤖 Balance of name (%s)" "$_wallet"

  yarn wicp:balance-of "$_wallet"

  wicpId="$(cd ./wicp && dfx canister id wicp)"

  printf "🤖 wICP Canister id is %s\n" "$wicpId"
}

deployCapRouter
[ "$DEBUG" == 1 ] && echo $?

# TODO: Check why it throws replica 404
# deployDab
# [ "$DEBUG" == 1 ] && echo $?

deployDip721 "$DEFAULT_USER_WALLET" "Crowns" "CRW"
[ "$DEBUG" == 1 ] && echo $?

deployMarketplace "$IC_HISTORY_ROUTER" "$DEFAULT_USER_WALLET"
[ "$DEBUG" == 1 ] && echo $?

deployWICP "$DEFAULT_USER_WALLET" "$IC_HISTORY_ROUTER"
[ "$DEBUG" == 1 ] && echo $?

echo "👍 Deploy services completed!"