#!/bin/bash

source "${BASH_SOURCE%/*}/.scripts/required/cap.sh"
source "${BASH_SOURCE%/*}/.scripts/required/default-identity.sh"
source "${BASH_SOURCE%/*}/.scripts/dfx-identity.sh"

marketplaceIcxPrologue="--candid=./marketplace/marketplace.did"
marketplaceId=""
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=""
fungibleContractAddress=""
icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)

deployWICP() {
  printf "🤖 Deploy wICP Token Canister\n"

  # args
  name="$1"
  owner="$2"
  pem="$3"
  token_id="$4"

  printf "🤖 token id (%s) is for (%s), (%s) \n" "$token_id" "$name" "$owner"

  yarn wicp:deploy "$owner"

  fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)

  printf "🤖 Balance of name (%s), address (%s) of token id (%s)" "$name" "$owner" "$token_id"

  yarn wicp:balance-of "$owner"

  (
    cd ./wicp || exit 1

    # Top-up Alice's account balance
    dfx canister --no-wallet \
      call wicp \
      transfer "(
        principal \"$ALICE_PRINCIPAL_ID\",
        1000000:nat
      )"

    printf "🤖 Balance of name (%s), address (%s) of token id (%s)" "Alice" "$ALICE_PRINCIPAL_ID" "$token_id"

    yarn wicp:balance-of "$ALICE_PRINCIPAL_ID"

    # Top-up Bob's account balance
    dfx canister --no-wallet \
      call wicp \
      transfer "(
        principal \"$BOB_PRINCIPAL_ID\",
        1000000:nat
      )"

    printf "🤖 Balance of name (%s), address (%s) of token id (%s)" "Bob" "$BOB_PRINCIPAL_ID" "$token_id"

    yarn wicp:balance-of "$BOB_PRINCIPAL_ID"
  )
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  yes yes | yarn marketplace:deploy "$icHistoryRouter" "$ownerPrincipalId"

  marketplaceId=$(dfx canister id marketplace)
}

deployNft() {
  printf "🤖 Deploy DIP721 NFT Canister\n"

  ownerPrincipalId=$(dfx identity get-principal)
  tokenSymbol="FOO"
  tokenName="Foobar"

  printf "🤖 Deploying NFT with %s %s %s\n" "$ownerPrincipalId" "$tokenSymbol" "$tokenName"

  yarn dip721:deploy-nft "$ownerPrincipalId" "$tokenSymbol" "$tokenName"

  yarn dip721:set-controllers

  nonFungibleContractAddress=$(cd ./DIP721 && dfx canister id nft)

  printf "NFT Contract address -> %s\n" "$nonFungibleContractAddress"
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  # Args
  name="$1"
  mint_for="$2"
  pem="$3"

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), mint_for user (%s) (%s)\n" "$nonFungibleContractAddress" "$name" "$mint_for"

  icx --pem="$pem" \
    update "$nonFungibleContractAddress" \
    mintDip721 "(
      principal \"$mint_for\",
      vec{}
    )" \
    --candid=./DIP721/nft/candid/nft.did

  printf "\n"
}

addCrownCollection() {
  echo "🤖 Add Crown Collection"

  printf "🤖 The addCrownCollection has marketplaceId (%s), nonFungibleContractAddress (%s), fungibleContractAddress (%s)\n" "$marketplaceId" "$nonFungibleContractAddress" "$fungibleContractAddress"

  icx --pem="$DEFAULT_PEM" \
    update "$marketplaceId" \
    addCollection "(
        principal \"$ownerPrincipalId\",
        10,
        0,
        \"Crowns Collection\",
        principal \"$nonFungibleContractAddress\",
        variant { DIP721 },
        principal \"$fungibleContractAddress\",
        variant { DIP20 }
      )" \
    "$marketplaceIcxPrologue"
}

listForSale() {
  echo "🤖 List for sale"

    token_id=0
    list_price="(12345:nat)"

    icx --pem="$DEFAULT_PEM" \
      update "$marketplaceId" \
      listForSale "(
          principal \"$nonFungibleContractAddress\",
          $token_id,
          $list_price
        )" \
      "$marketplaceIcxPrologue"
}

getSaleOffers() {
    printf "🤖 Call getSaleOffers\n"

    echo "🤖 marketplaceId is ($marketplaceId)"

    icx --pem="$DEFAULT_PEM" \
      query "$marketplaceId" \
      getSaleOffers "()" \
      "$marketplaceIcxPrologue"
}

makeBuyOffer() {
    printf "🤖 Call makeBuyOffer\n"

    token_id=0
    list_price="(321:nat)"

    icx --pem="$ALICE_PEM" \
      update "$marketplaceId" \
      makeBuyOffer "(
        principal \"$nonFungibleContractAddress\",
        $token_id,
        $list_price
      )" \
    "$marketplaceIcxPrologue"
}

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  deployWICP "Default" "$DEFAULT_PRINCIPAL_ID" "$DEFAULT_PEM" "wicp"
  deployMarketplace
  deployNft
  mintDip721 "default" "$DEFAULT_PRINCIPAL_ID" "$DEFAULT_PEM"
  addCrownCollection
  listForSale
  getSaleOffers
  makeBuyOffer
}

run

exit 0