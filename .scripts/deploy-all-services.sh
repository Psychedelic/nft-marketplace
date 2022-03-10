#!/bin/bash

set -x

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

DEBUG=1
DEFAULT_PRINCIPAL_ID=$(dfx identity get-principal)
IC_HISTORY_ROUTER=""

DEFAULT_USER_WALLET=$(dfx identity get-wallet)

echo "[debug] DEFAULT_USER_WALLET"
echo "$DEFAULT_USER_WALLET"

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

    ownerPrincipalId=$DEFAULT_USER_WALLET
    tokenSymbol="FOO"
    tokenName="Foobar"

    printf "🤖 Deploying NFT with owner id (%s), token (%s), token name (%s), cap (%s)\n" "$ownerPrincipalId" "$tokenSymbol" "$tokenName" "$IC_HISTORY_ROUTER"

    # yarn dip721:deploy-nft "local" "$ownerPrincipalId" "$tokenSymbol" "$tokenName" "$IC_HISTORY_ROUTER"

    # dfx canister --wallet "$DEFAULT_USER_WALLET" \
    #   create crowns \
    #   --controller "$ownerPrincipalId"

    # nonFungibleContractAddress=$(dfx canister id crowns)

    # printf "NFT Contract address -> %s\n" "$nonFungibleContractAddress"

    # dfx canister --wallet "$DEFAULT_USER_WALLET" \
    #   update-settings \
    #     --controller "$ownerPrincipalId" \
    #     --controller "$nonFungibleContractAddress" \
    #   "$nonFungibleContractAddress"

    # dfx deploy --wallet "$DEFAULT_USER_WALLET" \
    #   crowns --argument "(
    #     opt record {
    #       name = opt \"$tokenName\";
    #       logo = opt \"data:image/jpeg;base64,...\";
    #       symbol = opt \"$tokenSymbol\";
    #       owners = opt vec { principal \"$DEFAULT_USER_WALLET\" };
    #     }
    # )"

    # printf "Should copy the parent crowns wasm to the crowns directory"

    # cd .. || exit 1
    # targetDir=./crowns/target/wasm32-unknown-unknown/release
    # mkdir -p "$targetDir" || exit 1
    # cp ./target/wasm32-unknown-unknown/release/crowns.wasm "$targetDir" || exit 1

    dfx canister --wallet "$DEFAULT_USER_WALLET" \
      create crowns \
      --controller "$ownerPrincipalId"

    dfx build crowns

    nonFungibleContractAddress=$(dfx canister id crowns)

    dfx canister --wallet "$DEFAULT_USER_WALLET" \
      update-settings \
        --controller "$ownerPrincipalId" \
        --controller "$nonFungibleContractAddress" \
      "$nonFungibleContractAddress"

    dfx deploy --wallet "$DEFAULT_USER_WALLET" \
      crowns --argument "(
        opt record {
          name = opt \"$tokenName\";
          logo = opt \"data:image/jpeg;base64,...\";
          symbol = opt \"$tokenSymbol\";
          owners = opt vec { principal \"$DEFAULT_USER_WALLET\" };
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

deployMarketplace "$IC_HISTORY_ROUTER" "$DEFAULT_USER_WALLET"
[ "$DEBUG" == 1 ] && echo $?

deployWICP "$DEFAULT_USER_WALLET"
[ "$DEBUG" == 1 ] && echo $?

echo "👍 Deploy services completed!"