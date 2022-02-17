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
buy_offer_id=""

marketplaceIcxPrologue="--candid=./marketplace/marketplace.did"
wicpIcxPrologue="--candid=./wicp/src/wicp.did"
dip721IcxPrologue="--candid=./DIP721/nft/candid/nft.did"

updateControllers() {
  printf "🤖 Call updateControllers\n"

  _callerHome=$1
  _ownerPrincipalId=$2
  _nonFungibleContractAddress=$3

  printf "🤖 Set contract (%s) controller as (%s)\n" "$_nonFungibleContractAddress" "$_ownerPrincipalId"

  HOME=$_callerHome &&
  yarn dip721:set-controllers "$_ownerPrincipalId" "$_nonFungibleContractAddress"
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

  _callerHome=$1
  _wicpId=$2
  _marketplaceId=$3
  _amount=$4

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"

  # icx --pem="$BOB_PEM" \
  #   update "$wicpId" \
  #   approve "(
  #     principal \"$marketplaceId\",
  #     $_amount:nat
  #   )" \
  # "$wicpIcxPrologue"
}

topupWICP() {
  printf "🤖 Call topupWICP\n"

  _callerHome=$1
  _wicpId=$2
  _name=$3
  _transferTo=$4
  _amount=$5

  printf "🤖 Will top-up (%s) account id (%s) by 
  transfer of amount (%s) \n" "$_name" "$_transferTo" "$_amount"

  # icx --pem="$DEFAULT_PEM" \
  #   update "$wicpId" \
  #   transfer "(
  #     principal \"$_transferTo\",
  #     $_amount:nat
  #   )" \
  # "$wicpIcxPrologue"


  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call --update "$_wicpId" \
    transfer "(
      principal \"$_transferTo\",
      $_amount:nat
    )"

  printf "🤖 balance of (%s)\n" "$_name"

  yarn wicp:balance-of "$_transferTo"
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  _callerHome=$1
  _name=$2
  _mint_for=$3
  _nonFungibleContractAddress=$4

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), 
  mint_for user (%s) of id (%s)\n" "$_nonFungibleContractAddress" "$_name" "$_mint_for"

  _result=$(
    # TODO: When using DFX the error is thrown
    # The Replica returned an error: code 5, message: "Canister r7inp-6aaaa-aaaaa-aaabq-cai trapped explicitly: Custom(Fail to decode argument 1 from table0 to vec record
    # HOME=$_callerHome &&
    # dfx canister --no-wallet \
    #   call --update "$_nonFungibleContractAddress" \
    #   mintDip721 "(
    #     principal \"$_mint_for\",
    #     vec{}
    #   )"
    # TODO: While the ICX version works
    # Although the DFX version in the DIP721 works fine...
    icx --pem="$DEFAULT_PEM" \
    update "$_nonFungibleContractAddress" \
    mintDip721 "(
      principal \"$_mint_for\",
      vec{}
    )" \
    --candid=./DIP721/nft/candid/nft.did
  )

  nft_token_id_for_alice=$(echo "$_result" | pcregrep -o1  'token_id = ([0-9]*)')

  printf "🤖 Minted Dip721 for user %s, has token ID (%s)\n" "$_name" "$nft_token_id_for_alice"

  printf "🤖 The Balance for user %s of id (%s)\n" "$_name" "$_mint_for"

  # icx --pem="$DEFAULT_PEM" \
  #   query "$nonFungibleContractAddress" \
  #   balanceOfDip721 "(
  #     principal \"$mint_for\"
  #   )" \
  #   --candid=./DIP721/nft/candid/nft.did

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call "$_nonFungibleContractAddress" \
    balanceOfDip721 "(
      principal \"$_mint_for\",
      vec{}
    )"

  printf "🤖 User %s getMetadataForUserDip721 is\n" "$_name"

  # icx --pem="$DEFAULT_PEM" \
  #   query "$nonFungibleContractAddress" \
  #   getMetadataForUserDip721 "(
  #     principal \"$mint_for\"
  #   )" \
  #   --candid=./DIP721/nft/candid/nft.did

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call "$nonFungibleContractAddress" \
    getMetadataForUserDip721 "(
      principal \"$_mint_for\",
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

  _callerHome=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _nft_token_id_for_alice=$4

  printf "🤖 Default approves Marketplace id (%s)\n 
  for non-fungible contract address (%s) 
  the token id (%s)" "$_marketplaceId"  "$_nonFungibleContractAddress" "$_nft_token_id_for_alice"

  # HOME=$_callerHome &&
  # dfx canister --no-wallet \
  #   call "$_nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$_marketplaceId\",
  #     $_nft_token_id_for_alice:nat64
  #   )"

  icx --pem="$DEFAULT_PEM" \
    update "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      $_nft_token_id_for_alice:nat64
    )" \
  "$dip721IcxPrologue"
}

addCrownCollection() {
  echo "🤖 Add Crown Collection"

  _callerHome=$1
  _ownerPrincipalId=$2
  _marketplaceId=$3
  _nonFungibleContractAddress=$4
  _fungibleContractAddress=$5
  _collectionName=$6
  _fee=$7
  _creationTime=$8

  printf "🤖 The addCrownCollection has owner id (%s) marketplaceId (%s), 
  nonFungibleContractAddress (%s), 
  fungibleContractAddress (%s)\n" "$_ownerPrincipalId" "$_marketplaceId" "$_nonFungibleContractAddress" "$_fungibleContractAddress"

  printf "🤖 collection name (%s), fee (%s) and creation time (%s)\n" "$_collectionName" "$_fee" "$_creationTime"

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

  # Interface
  # owner: Principal,
  # owner_fee_percentage: u16,
  # creation_time: u64,
  # collection_name: String,
  # non_fungible_contract_address: Principal,
  # non_fungible_token_type: NonFungibleTokenType,
  # fungible_contract_address: Principal,
  # fungible_token_type: FungibleTokenType,

  # HOME=$_callerHome &&
  # dfx canister --no-wallet \
  #   call --update "$_marketplaceId" \
  #   addCollection "(
  #       principal \"$_ownerPrincipalId\",
  #       $_fee,
  #       $_creationTime,
  #       \"$_collectionName\",
  #       principal \"$_nonFungibleContractAddress\",
  #       variant { DIP721 },
  #       principal \"$_fungibleContractAddress\",
  #       variant { DIP20 }
  #     )"
}

listForSale() {
  printf "🤖 List for sale\n"

  _callerHome=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4
  _list_price=$5

  printf "🤖 has market id (%s)\n" "$_marketplaceId"
  printf "🤖 the token id is %s, price %s\n" "$_token_id" "$_list_price"

  # icx --pem="$caller_pem" \
  #   update "$marketplaceId" \
  #   listForSale "(
  #       principal \"$nonFungibleContractAddress\",
  #       $token_id,
  #       $list_price
  #     )" \
  #   "$marketplaceIcxPrologue"

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call --update "$_marketplaceId" \
    listForSale "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
        ($_list_price:nat)
      )"

  # icx --pem="$ALICE_PEM" \
  #   update "$marketplaceId" \
  #   listForSale "(
  #       principal \"$nonFungibleContractAddress\",
  #       $_token_id,
  #       $_list_price
  #     )" \
  #   "$marketplaceIcxPrologue"
}

getSaleOffers() {
    printf "🤖 Call getSaleOffers\n"

  _callerHome=$1
  _marketplaceId=$2

    # icx --pem="$DEFAULT_PEM" \
    #   query "$marketplaceId" \
    #   getSaleOffers "()" \
    #   "$marketplaceIcxPrologue"

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call --query "$_marketplaceId" \
    getSaleOffers "()"

  # icx --pem="$DEFAULT_PEM" \
  #   query "$marketplaceId" \
  #   getSaleOffers "()" \
  #   "$marketplaceIcxPrologue"
}

makeBuyOffer() {
    printf "🤖 Call makeBuyOffer\n"

    _callerHome=$1
    _marketplaceId=$2
    _name=$3
    _token_id=$4
    _offer_price=$5

    printf "🤖 The market id (%s)" "$_marketplaceId"
    printf "🤖 (%s) will makeBuyOffer for token id (%s) 
    for the amount (%s)\n" "$_name" "$_token_id" "$_offer_price"

  printf "🤖 balance of (%s) is equal to\n" "$_name"

  yarn wicp:balance-of "$_transferTo"

  # icx --pem="$BOB_PEM" \
  #   update "$marketplaceId" \
  #   makeBuyOffer "(
  #     principal \"$nonFungibleContractAddress\",
  #     $_token_id,
  #     $_offer_price
  #   )" \
  # "$marketplaceIcxPrologue"

  _result=$(
    HOME=$_callerHome &&
    dfx canister --no-wallet \
      call --update "$_marketplaceId" \
      makeBuyOffer "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
        ($_offer_price:nat)
      )"
  )

  # _result=$(
  #   icx --pem="$BOB_PEM" \
  #     update "$marketplaceId" \
  #     makeBuyOffer "(
  #       principal \"$nonFungibleContractAddress\",
  #       $_token_id,
  #       $_offer_price
  #     )" \
  #   "$marketplaceIcxPrologue"
  # )

  #
  _result_num=$(echo "$_result" | pcregrep -o1  'Ok = ([0-9]*)')

  printf "🤖 Extracted the latest index in the offers (%s)\n" "$_result_num"

  buy_offer_id=$((_result_num - 1))

  printf "🤖 (%s) offer id is (%s)\n" "$_name" "$buy_offer_id"
}

getBuyOffers() {
    printf "🤖 Call getBuyOffers\n"

    _callerHome=$1
    _marketplaceId=$2
    _begin=$3
    _limit=$4
  
    printf "🤖 The getBuyOffers from marketplace id (%s) 
    was called with being (%s) and limit (%s)\n" "$_marketplaceId" "$_begin" "$_limit"

    HOME=$_callerHome &&
    dfx canister --no-wallet \
      call --query "$_marketplaceId" \
      getBuyOffers "($_begin, $_limit)"
}

approveTransferFromForAcceptBuyOffer() {
  printf "🤖 Call approveTransferFromForAcceptBuyOffer\n"  

  _callerHome=$1
  _name=$2
  _nonFungibleContractAddress=$3
  _marketplaceId=$4
  _nft_token_id_for_alice=$5

  printf "🤖 The user (%s) will approve transfer token id (%s) 
  for marketplace id (%s) \n" "$_name" "$_nft_token_id_for_alice" "$_marketplaceId"
  printf "🤖 for nft contract id (%s)" "$_nonFungibleContractAddress"

  # icx --pem="$sale_owner_pem" \
  #   update "$nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$marketplaceId\",
  #     $nft_token_id_for_alice:nat64
  #   )" \
  # "$dip721IcxPrologue"  

  # HOME=$_callerHome &&
  # dfx canister --no-wallet \
  #   call --update "$_nonFungibleContractAddress" \
  #   approveDip721 "(
  #     principal \"$_marketplaceId\",
  #     $_nft_token_id_for_alice:nat64
  #   )"

  icx --pem="$ALICE_PEM" \
    update "$nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$marketplaceId\",
      $nft_token_id_for_alice:nat64
    )" \
  "$dip721IcxPrologue"  
}

acceptBuyOffer() {
    printf "🤖 Call acceptBuyOffer\n"

    _callerHome=$1
    _marketplaceId=$2
    _buy_offer_id=$3

    # icx --pem="$ALICE_PEM" \
    #   update "$_marketplaceId" \
    #   acceptBuyOffer "($_buy_offer_id)" \
    #  "$marketplaceIcxPrologue"

  HOME=$_callerHome &&
  dfx canister --no-wallet \
    call --update "$_marketplaceId" \
    acceptBuyOffer "($_buy_offer_id:nat64)"

  # icx --pem="$ALICE_PEM" \
  #   update "$_marketplaceId" \
  #   acceptBuyOffer "($_buy_offer_id)" \
  #   "$marketplaceIcxPrologue"
}

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  updateControllers "$HOME" "$DEFAULT_PRINCIPAL_ID" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  topupWICP "$HOME" "$wicpId" "Bob" "$BOB_PRINCIPAL_ID" "50_000_000"
  [ "$DEBUG" == 1 ] && echo $?
  
  allowancesForWICP "$BOB_HOME" "$wicpId" "$marketplaceId" "100_000_000"
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 "$HOME" "Alice" "$ALICE_PRINCIPAL_ID" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  # TODO: allowances Throws "other" error
  # allowancesForDIP721 "$HOME" "$nonFungibleContractAddress" "$marketplaceId" "$nft_token_id_for_alice"
  # [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection "$HOME" "$DEFAULT_PRINCIPAL_ID" "$marketplaceId" "$nonFungibleContractAddress" "$fungibleContractAddress" "Crowns Collection" 10 0
  [ "$DEBUG" == 1 ] && echo $?

  listForSale "$ALICE_HOME" "$nonFungibleContractAddress" "$marketplaceId" "$nft_token_id_for_alice" "1_250"
  [ "$DEBUG" == 1 ] && echo $?

  getSaleOffers "$HOME" "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  makeBuyOffer "$BOB_HOME" "$marketplaceId" "Bob" "$nft_token_id_for_alice" "1_000"
  [ "$DEBUG" == 1 ] && echo $?

  getBuyOffers "$HOME" "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  # TODO: allowances Throws "other" error (variant { Err = variant { Other } })
  # approveTransferFromForAcceptBuyOffer "$ALICE_HOME" "Alice" "$nonFungibleContractAddress" "$marketplaceId" "$nft_token_id_for_alice"
  # [ "$DEBUG" == 1 ] && echo $?

  acceptBuyOffer "$ALICE_HOME" "$marketplaceId" "$buy_offer_id"
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "👍 Healthcheck completed!"