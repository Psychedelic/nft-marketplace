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

deployToken() {
  printf "🤖 Deploy DIP20 Token Canister\n"

  yarn dip20:deploy

  fungibleContractAddress=$(cd ./DIP20/motoko && dfx canister id token)
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  mint_for="$ownerPrincipalId"

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), mint_for (%s)\n" "$nonFungibleContractAddress" "$mint_for"

  icx --pem="$DEFAULT_PEM" \
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
    list_price="(123:nat)"

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

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  deployToken
  deployMarketplace
  deployNft
  mintDip721
  addCrownCollection
  listForSale
  getSaleOffers
}

run

exit 0