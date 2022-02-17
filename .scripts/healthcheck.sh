#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

. ".scripts/required/cap.sh"
. ".scripts/required/default-identity.sh"
. ".scripts/dfx-identity.sh"

marketplaceId=$(dfx canister id marketplace)
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=$(cd ./DIP721 && dfx canister id nft)
fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)
icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)
wicpId="$(cd ./wicp && dfx canister id wicp)"
nft_token_id_for_alice=""

dip721IcxPrologue="--candid=./DIP721/nft/candid/nft.did"

updateControllers() {
  printf "🤖 Call updateControllers\n"

  callerHome=$1
  ownerPrincipalId=$2
  nonFungibleContractAddress=$3

  printf "🤖 Set contract (%s) controller as (%s)\n" "$nonFungibleContractAddress" "$ownerPrincipalId"

  HOME=$callerHome &&
  yarn dip721:set-controllers "$ownerPrincipalId" "$nonFungibleContractAddress"
}

allowancesForWICP() {
  printf "🤖 Call allowancesForWICP\n"

  printf "🤖 Bob approves Marketplace (%s)\n" "$marketplaceId"

  # icx --pem="$BOB_PEM" \
  #   update "$wicpId" \
  #   approve "(
  #     principal \"$marketplaceId\",
  #     10_000_000:nat
  #   )" \
  # "$wicpIcxPrologue"

  callerHome=$1
  marketplaceId=$2

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$wicpId" \
    approve "(
      principal \"$marketplaceId\",
      10_000_000:nat
    )"
}

topupWICP() {
  printf "🤖 Call topupWICP\n"

  callerHome=$1
  wicpId=$2
  name=$3
  transferTo=$4
  amount=$5

  printf "🤖 Will top-up Bob's account by transfer %s\n" "$name"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$wicpId" \
    transfer "(
      principal \"$transferTo\",
      $amount:nat
    )"

  printf "🤖 balance of Bob\n"

  yarn wicp:balance-of "$transferTo"
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  callerHome=$1
  name=$2
  mint_for=$3
  nonFungibleContractAddress=$4

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), 
  mint_for user (%s) of id (%s)\n" "$nonFungibleContractAddress" "$name" "$mint_for"

  result=$(
    # TODO: When using DFX the error is thrown
    # The Replica returned an error: code 5, message: "Canister r7inp-6aaaa-aaaaa-aaabq-cai trapped explicitly: Custom(Fail to decode argument 1 from table0 to vec record
    # HOME=$callerHome &&
    # dfx canister --no-wallet \
    #   call "$nonFungibleContractAddress" \
    #   mintDip721 "(
    #     principal \"$mint_for\",
    #     vec{}
    #   )"
    # TODO: While the ICX version works
    # Although the DFX version in the DIP721 works fine...
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

  # icx --pem="$DEFAULT_PEM" \
  #   query "$nonFungibleContractAddress" \
  #   balanceOfDip721 "(
  #     principal \"$mint_for\"
  #   )" \
  #   --candid=./DIP721/nft/candid/nft.did

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$nonFungibleContractAddress" \
    balanceOfDip721 "(
      principal \"$mint_for\",
      vec{}
    )"

  printf "🤖 User %s getMetadataForUserDip721 is\n" "$name"

  # icx --pem="$DEFAULT_PEM" \
  #   query "$nonFungibleContractAddress" \
  #   getMetadataForUserDip721 "(
  #     principal \"$mint_for\"
  #   )" \
  #   --candid=./DIP721/nft/candid/nft.did

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$nonFungibleContractAddress" \
    getMetadataForUserDip721 "(
      principal \"$mint_for\",
      vec{}
    )"

  printf "\n"
}

deployMarketplace() {
  printf "🤖 Call the deployMarketplace\n"

  yes yes | yarn marketplace:deploy "$icHistoryRouter" "$ownerPrincipalId"

  marketplaceId=$(dfx canister id marketplace)
}

# TODO: Throwing error (variant { Err = variant { Other } })
allowancesForDIP721() {
  printf "🤖 Call the allowancesForDIP721\n"

  # icx --pem="$DEFAULT_PEM" \
  #   update "$nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$marketplaceId\",
  #     0:nat64
  #   )" \
  # "$dip721IcxPrologue"

  callerHome=$1
  nonFungibleContractAddress=$2
  marketplaceId=$3
  amount=$4

  printf "🤖 Default approves Marketplace id (%s)\n 
  for non-fungible contract address (%s) 
  the amount (%s)" "$marketplaceId"  "$nonFungibleContractAddress" "$amount"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      $amount:nat64
    )"

  # icx --pem="$DEFAULT_PEM" \
  #   update "$nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$marketplaceId\",
  #     0:nat64
  #   )" \
  # "$dip721IcxPrologue"
}

addCrownCollection() {
  echo "🤖 Add Crown Collection"

  callerHome=$1
  marketplaceId=$2
  nonFungibleContractAddress=$3
  fungibleContractAddress=$4
  collectionName=$5
  fee=$6
  creationTime=$7

  printf "🤖 The addCrownCollection has marketplaceId (%s), nonFungibleContractAddress (%s), fungibleContractAddress (%s)\n" "$marketplaceId" "$nonFungibleContractAddress" "$fungibleContractAddress"
  printf "🤖 collection name (%s), fee (%s) and creation time (%s)\n" "$collectionName" "$fee" "$creationTime"

  # icx --pem="$DEFAULT_PEM" \
  #   update "$marketplaceId" \
  #   addCollection "(
  #       principal \"$ownerPrincipalId\",
  #       10,
  #       0,
  #       \"Crowns Collection\",
  #       principal \"$nonFungibleContractAddress\",
  #       variant { DIP721 },
  #       principal \"$fungibleContractAddress\",
  #       variant { DIP20 }
  #     )" \
  #   "$marketplaceIcxPrologue"

  # Interface
  # owner: Principal,
  # owner_fee_percentage: u16,
  # creation_time: u64,
  # collection_name: String,
  # non_fungible_contract_address: Principal,
  # non_fungible_token_type: NonFungibleTokenType,
  # fungible_contract_address: Principal,
  # fungible_token_type: FungibleTokenType,

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$marketplaceId" \
    addCollection "(
        principal \"$ownerPrincipalId\",
        $fee,
        $creationTime,
        \"$collectionName\",
        principal \"$nonFungibleContractAddress\",
        variant { DIP721 },
        principal \"$fungibleContractAddress\",
        variant { DIP20 }
      )"
}

listForSale() {
  printf "🤖 List for sale\n"

  callerHome=$1
  nonFungibleContractAddress=$2
  token_id=$3
  list_price=$4

  printf "🤖 the token id is %s, price %s\n" "$token_id" "$list_price"

  # icx --pem="$caller_pem" \
  #   update "$marketplaceId" \
  #   listForSale "(
  #       principal \"$nonFungibleContractAddress\",
  #       $token_id,
  #       $list_price
  #     )" \
  #   "$marketplaceIcxPrologue"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$marketplaceId" \
    listForSale "(
        principal \"$nonFungibleContractAddress\",
        $token_id,
        ($list_price:nat)
      )"
}

getSaleOffers() {
    printf "🤖 Call getSaleOffers\n"

  callerHome=$1
  marketplaceId=$2

    # icx --pem="$DEFAULT_PEM" \
    #   query "$marketplaceId" \
    #   getSaleOffers "()" \
    #   "$marketplaceIcxPrologue"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$marketplaceId" \
    getSaleOffers "()"
}

makeBuyOffer() {
    printf "🤖 Call makeBuyOffer\n"

    callerHome=$1
    name=$2
    token_id=$3
    offer_price=$4

    printf "🤖 (%s) will makeBuyOffer for token id (%s) for the amount (%s)\n" "$name" "$token_id" "$offer_price"

    # icx --pem="$offer_from_pem" \
    #   update "$marketplaceId" \
    #   makeBuyOffer "(
    #     principal \"$nonFungibleContractAddress\",
    #     $token_id,
    #     $offer_price
    #   )" \
    # "$marketplaceIcxPrologue"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$marketplaceId" \
    makeBuyOffer "(
      principal \"$nonFungibleContractAddress\",
      $token_id,
      ($offer_price:nat)
    )"
}

getBuyOffers() {
    printf "🤖 Call getBuyOffers\n"

    callerHome=$1
    marketplaceId=$2
    begin=$3
    limit=$4
  
    printf "🤖 The getBuyOffers from marketplace id (%s) was called with being (%s) and limit (%s)\n" "$marketplaceId" "$begin" "$limit"

    HOME=$callerHome &&
    dfx canister --no-wallet \
      call "$marketplaceId" \
      getBuyOffers "($begin, $limit)"
}

approveTransferFromForAcceptBuyOffer() {
  printf "🤖 Call approveTransferFromForAcceptBuyOffer\n"  

  callerHome=$1
  name=$2
  marketplaceId=$3
  amount=$4

  printf "🤖 The user (%s) will approve transfer amount (%s) 
  from marketplace id (%s) \n" "$name" "$amount" "$marketplaceId"

  # icx --pem="$sale_owner_pem" \
  #   update "$nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$marketplaceId\",
  #     $amount:nat64
  #   )" \
  # "$dip721IcxPrologue"  

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      $amount:nat64
    )"
}

acceptBuyOffer() {
    printf "🤖 Call acceptBuyOffer\n"

    callerHome=$1
    marketplaceId=$2
    buy_id=$3

    # icx --pem="$sale_owner_pem" \
    #   update "$marketplaceId" \
    #   acceptBuyOffer "($buy_id)" \
    #  "$marketplaceIcxPrologue"

  HOME=$callerHome &&
  dfx canister --no-wallet \
    call "$marketplaceId" \
    acceptBuyOffer "($buy_id)"
}

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  updateControllers "$HOME" "$DEFAULT_PRINCIPAL_ID" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForWICP "$HOME" "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  topupWICP "$HOME" "$wicpId" "Bob" "$BOB_PRINCIPAL_ID" "5_000_000"
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 "$HOME" "Alice" "$ALICE_PRINCIPAL_ID" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForDIP721 "$HOME" "$nonFungibleContractAddress" "$marketplaceId" 0
  [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection "$HOME" "$marketplaceId" "$nonFungibleContractAddress" "$fungibleContractAddress" "Crowns Collection" 10 0
  [ "$DEBUG" == 1 ] && echo $?

  listForSale "$ALICE_HOME" "$marketplaceId" "$nft_token_id_for_alice" "1_250"
  [ "$DEBUG" == 1 ] && echo $?

  getSaleOffers "$HOME" "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  makeBuyOffer "$HOME_BOB" "Bob" "$nft_token_id_for_alice" "1_000"
  [ "$DEBUG" == 1 ] && echo $?

  getBuyOffers "$HOME" "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForAcceptBuyOffer "$ALICE_HOME" "Alice" "$marketplaceId" "50_000"
  [ "$DEBUG" == 1 ] && echo $?

  acceptBuyOffer "$ALICE_HOME" "$marketplaceId" 0
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "👍 Healthcheck completed!"