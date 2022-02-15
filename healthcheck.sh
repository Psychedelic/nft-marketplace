#!/bin/bash

[ "$DEBUG" == 1 ] && set -x

source "${BASH_SOURCE%/*}/.scripts/required/cap.sh"
source "${BASH_SOURCE%/*}/.scripts/required/default-identity.sh"
source "${BASH_SOURCE%/*}/.scripts/dfx-identity.sh"

dip721IcxPrologue="--candid=./DIP721/nft/candid/nft.did"
wicpIcxPrologue="--candid=./wicp/src/wicp.did"
marketplaceIcxPrologue="--candid=./marketplace/marketplace.did"
marketplaceId=$(dfx canister id marketplace)
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=$(cd ./DIP721 && dfx canister id nft)
fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)
icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)
wicpId="$(cd ./wicp && dfx canister id wicp)"
nft_token_id_for_alice=""

allowancesForWICP() {
  printf "🤖 Call allowancesForWICP\n"

  printf "🤖 Bob approves Marketplace (%s)\n" "$marketplaceId"

  icx --pem="$BOB_PEM" \
    update "$wicpId" \
    approve "(
      principal \"$marketplaceId\",
      10_000_000:nat
    )" \
  "$wicpIcxPrologue"
}

topupWICP() {
  printf "🤖 Call topupWICP\n"

  printf "🤖 Will top-up Bob's account by transfer %s\n" "$BOB_PRINCIPAL_ID"

  dfx canister --no-wallet \
    call "$wicpId" \
    transfer "(
      principal \"$BOB_PRINCIPAL_ID\",
      5_000_000:nat
    )"

  printf "🤖 balance of Bob\n"

  yarn wicp:balance-of "$BOB_PRINCIPAL_ID"
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  yes yes | yarn marketplace:deploy "$icHistoryRouter" "$ownerPrincipalId"

  marketplaceId=$(dfx canister id marketplace)
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  # Args
  name="$1"
  mint_for="$2"

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), mint_for user (%s) (%s)\n" "$nonFungibleContractAddress" "$name" "$mint_for"

  result=$(
    icx --pem="$DEFAULT_PEM" \
    update "$nonFungibleContractAddress" \
    mintDip721 "(
      principal \"$mint_for\",
      vec{}
    )" \
    --candid=./DIP721/nft/candid/nft.did
  )

  nft_token_id_for_alice=$(echo "$result" | pcregrep -o1  'token_id = ([0-9]*)')

  printf "🤖 Minted Dip721 for user %s, has token ID (%s)\n" "$name" "$nft_token_id_for_alice"

  printf "🤖 The Balance for user %s of id (%s)\n" "$name" "$mint_for"

  icx --pem="$DEFAULT_PEM" \
    query "$nonFungibleContractAddress" \
    balanceOfDip721 "(
      principal \"$mint_for\"
    )" \
    --candid=./DIP721/nft/candid/nft.did

  printf "🤖 User %s getMetadataForUserDip721 is\n" "$name"

  icx --pem="$DEFAULT_PEM" \
    query "$nonFungibleContractAddress" \
    getMetadataForUserDip721 "(
      principal \"$mint_for\"
    )" \
    --candid=./DIP721/nft/candid/nft.did

  printf "\n"
}

# TODO: Throwing error (variant { Err = variant { Other } })
allowancesForDIP721() {
  printf "🤖 Call the allowancesForDIP721\n"
  printf "🤖 Default approves Marketplace (%s)\n for non-fungible contract address (%s)" "$marketplaceId" "$nonFungibleContractAddress"

  icx --pem="$DEFAULT_PEM" \
    update "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      0:nat64
    )" \
  "$dip721IcxPrologue"
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
  printf "🤖 List for sale\n"

  caller_pem=$1
  token_id=$2
  list_price=$3

  printf "🤖 the token id is %s, price %s\n" "$token_id" "$list_price"

  icx --pem="$caller_pem" \
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

    icx --pem="$DEFAULT_PEM" \
      query "$marketplaceId" \
      getSaleOffers "()" \
      "$marketplaceIcxPrologue"
}

makeBuyOffer() {
    printf "🤖 Call makeBuyOffer\n"

    name=$1
    offer_from_pem=$2
    token_id=$3
    offer_price=$4

    printf "🤖 %s will makeBuyOffer for token id %s\n" "$name" "$token_id"

    icx --pem="$offer_from_pem" \
      update "$marketplaceId" \
      makeBuyOffer "(
        principal \"$nonFungibleContractAddress\",
        $token_id,
        $offer_price
      )" \
    "$marketplaceIcxPrologue"
}

getBuyOffers() {
    printf "🤖 Call getBuyOffers\n"

    begin=$1
    limit=$2
  
    printf "🤖 The getBuyOffers was called with being (%s) and limit (%s)\n" "$begin" "$limit"

    dfx canister --no-wallet \
      call "$marketplaceId" \
      getBuyOffers "($begin, $limit)"
}

approveTransferFromForAcceptBuyOffer() {
  printf "🤖 Call approveTransferFromForAcceptBuyOffer\n"  

  sale_owner_pem=$1
  nft_token_id_for_alice=$2

  printf "🤖 The user %s will approve marketplace (%s) \n" "$sale_owner_pem" "$marketplaceId"

  icx --pem="$sale_owner_pem" \
    update "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      $nft_token_id_for_alice:nat64
    )" \
  "$dip721IcxPrologue"  
}

acceptBuyOffer() {
    printf "🤖 Call acceptBuyOffer\n"

    sale_owner_pem=$1
    buy_id=$2

    icx --pem="$sale_owner_pem" \
      update "$marketplaceId" \
      acceptBuyOffer "($buy_id)" \
     "$marketplaceIcxPrologue"
}

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  allowancesForWICP
  [ "$DEBUG" == 1 ] && echo $?

  topupWICP
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 "Alice" "$ALICE_PRINCIPAL_ID"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForDIP721
  [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection
  [ "$DEBUG" == 1 ] && echo $?

  listForSale "$ALICE_PEM" "$nft_token_id_for_alice" "(1_250:nat)"
  [ "$DEBUG" == 1 ] && echo $?

  getSaleOffers
  [ "$DEBUG" == 1 ] && echo $?

  makeBuyOffer "Bob" "$BOB_PEM" "$nft_token_id_for_alice" "(1_000:nat)"
  [ "$DEBUG" == 1 ] && echo $?

  getBuyOffers 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForAcceptBuyOffer "$ALICE_PEM" "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  acceptBuyOffer "$ALICE_PEM" 0
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "👍 Healthcheck completed!"