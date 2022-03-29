#!/bin/bash

# set -x

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

set -e

. ".scripts/required/cap.sh"
. ".scripts/dfx-identity.sh"

marketplaceId=$(dfx canister id marketplace)
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=$(cd ./crowns && dfx canister id crowns)
fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)
wicpId="$(cd ./wicp && dfx canister id wicp)"
nft_token_id_for_alice=""
buy_offer_id=""

topupWICP() {
  printf "🤖 Call topupWICP\n"

  _wicpId=$1
  _name=$2
  _transferTo=$3
  _amount=$4

  printf "🤖 Will top-up (%s) account id (%s) by 
  transfer of amount (%s) \n" "$_name" "$_transferTo" "$_amount"

  dfx canister \
    call --update "$_wicpId" \
    transfer "(
      principal \"$_transferTo\",
      $_amount:nat
    )"

  printf "🤖 balance of (%s)\n" "$_name"

  # TODO: wicp:balance-of refactor or remove
  # yarn wicp:balance-of "$_transferTo"

  dfx canister \
    call --query "$_wicpId" \
    balanceOf "(
      principal \"$_transferTo\"
    )"
}

allowancesForWICP() {
  printf "🤖 Call allowancesForWICP\n"

  printf "🤖 Bob approves Marketplace (%s)\n" "$marketplaceId"

  _wicpId=$1
  _marketplaceId=$2
  _amount=$3

  dfx canister \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  _name=$1
  _mint_for=$2
  _nonFungibleContractAddress=$3

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), 
  mint_for user (%s) of id (%s)\n" "$_nonFungibleContractAddress" "$_name" "$_mint_for"

  nft_token_id_for_alice=$RANDOM

  _result=$(
    dfx canister \
      call --update "$_nonFungibleContractAddress" \
      mint "(
        principal \"$_mint_for\",
        $nft_token_id_for_alice:nat,
        vec {
          record {
            \"location\";
            variant {
              \"TextContent\" = \"https://vqcq7-gqaaa-aaaam-qaara-cai.raw.ic0.app/0000.mp4\"
            }
          }
        }
      )"
  )

  printf "🤖 Minted Dip721 for user %s, has token ID (%s)\n" "$_name" "$nft_token_id_for_alice"

  printf "🤖 The Balance for user %s of id (%s)\n" "$_name" "$_mint_for"

  dfx canister \
    call "$_nonFungibleContractAddress" \
    balanceOf "(
      principal \"$_mint_for\",
    )"

  printf "🤖 User %s ownerTokenMetadata is\n" "$_name"

  dfx canister \
    call "$_nonFungibleContractAddress" \
    ownerTokenMetadata "(
      principal \"$_mint_for\",
    )"

  printf "\n"
}

allowancesForDIP721() {
  printf "🤖 Call the allowancesForDIP721\n"

  _nonFungibleContractAddress=$1
  _marketplaceId=$2
  _nft_token_id_for_alice=$3

  printf "🤖 Default approves Marketplace id (%s)\n 
  for non-fungible contract address (%s) 
  the token id (%s)" "$_marketplaceId"  "$_nonFungibleContractAddress" "$_nft_token_id_for_alice"

  dfx canister \
    call "$_nonFungibleContractAddress" \
    approve "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat
    )"
}

addCrownCollection() {
  echo "🤖 Add Crown Collection"

  _ownerPrincipalId=$1
  _marketplaceId=$2
  _nonFungibleContractAddress=$3
  _fungibleContractAddress=$4
  _collectionName=$5
  _fee=$6
  _creationTime=$7

  printf "🤖 The addCrownCollection has owner id (%s) marketplaceId (%s), 
  nonFungibleContractAddress (%s), 
  fungibleContractAddress (%s)\n" "$_ownerPrincipalId" "$_marketplaceId" "$_nonFungibleContractAddress" "$_fungibleContractAddress"

  printf "🤖 collection name (%s), fee (%s) and creation time (%s)\n" "$_collectionName" "$_fee" "$_creationTime"

  # Interface
  # owner: Principal,
  # owner_fee_percentage: u16,
  # creation_time: u64,
  # collection_name: String,
  # non_fungible_contract_address: Principal,
  # non_fungible_token_type: NonFungibleTokenType,
  # fungible_contract_address: Principal,
  # fungible_token_type: FungibleTokenType,

  dfx canister \
    call --update "$_marketplaceId" \
    addCollection "(
        principal \"$_ownerPrincipalId\",
        ($_fee:nat16),
        ($_creationTime:nat64),
        \"$_collectionName\",
        principal \"$_nonFungibleContractAddress\",
        variant { DIP721 },
        principal \"$_fungibleContractAddress\",
        variant { DIP20 }
      )"
}

makeListing() {
  printf "🤖 Call makeListing\n"

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4
  _list_price=$5

  printf "🤖 has market id (%s)\n" "$_marketplaceId"
  printf "🤖 the token id is %s, price %s\n" "$_token_id" "$_list_price"
  printf "🤖 will use identity %s\n" "$_identityName"

  dfx --identity "$_identityName" \
    canister call --update "$_marketplaceId" \
    makeListing "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
        $_list_price:nat
      )"
}

getAllListings() {
    printf "🤖 Call getAllListings\n"

  _marketplaceId=$1

  dfx canister \
    call --query "$_marketplaceId" \
    getAllListings "()"
}

makeOffer() {
  printf "🤖 Call makeOffer\n"

  _identityName=$1
  _name=$2
  _nonFungibleContractAddress=$3
  _token_id=$4
  _offer_price=$5

  printf "🤖 The user (%s) will makeOffer for token id (%s) 
  for the amount (%s)\n" "$_name" "$_token_id" "$_offer_price"

  printf "🤖 balance of (%s) is equal to\n" "$_name"
  yarn wicp:balance-of "$_transferTo"

  _result=$(
    dfx --identity "$_identityName" \
      canister call --update "$_marketplaceId" \
      makeOffer "(
        principal \"$_nonFungibleContractAddress\",
        ($_token_id:nat64),
        ($_offer_price:nat)
      )"
  )

  _result_num=$(echo "$_result" | pcregrep -o1  'Ok = ([0-9]*)')

  printf "🤖 Extracted the latest index in the offers (%s)\n" "$_result_num"

  buy_offer_id=$((_result_num - 1))

  printf "🤖 (%s) offer id is (%s)\n" "$_name" "$buy_offer_id"
}

getAllOffers() {
  printf "🤖 Call getAllOffers\n"

  _marketplaceId=$1
  _begin=$2
  _limit=$3

  printf "🤖 The getAllOffers from marketplace id (%s) 
  was called with being (%s) and limit (%s)\n" "$_marketplaceId" "$_begin" "$_limit"

  dfx canister \
    call --query "$_marketplaceId" \
    getAllOffers "($_begin, $_limit)"
}

approveTransferFromForNFTAcceptOffer() {
  printf "🤖 Call approveTransferFromForNFTAcceptOffer\n"  

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _nft_token_id_for_alice=$4
  _wicpId=$5

  printf "🤖 The user (%s) will approve transfer token id (%s) 
  for marketplace (%s) \n" "$_identityName" "$_nft_token_id_for_alice" "$_marketplaceId"
  printf "🤖 for nft contract id (%s)\n" "$_nonFungibleContractAddress"

  dfx --identity "$_identityName" \
    canister \
    call --update "$_nonFungibleContractAddress" \
    approve "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat
    )"
}


approveTransferFromForWICPAcceptOffer() {
  printf "🤖 Call approveTransferFromForWICPAcceptOffer\n"

  _identityName=$1
  _wicpId=$2
  _marketplaceId=$3
  _amount=$4

  printf "🤖 The user (%s) approves (%s) for the amount (%s) in fungible token (%s)\n" "$_identityName" "$_marketplaceId" "$_amount" "$_wicpId"

  dfx --identity "$_identityName" \
    canister \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"
}

acceptOffer() {
  printf "🤖 Call acceptOffer\n"

  _identityName=$1
  _marketplaceId=$2
  _buy_offer_id=$3

  dfx --identity "$_identityName" \
    canister \
    call --update "$_marketplaceId" \
    acceptOffer "($_buy_offer_id:nat64)"
}

run() {
  printf "🚑 Healthcheck runtime details\n"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  topupWICP \
    "$wicpId" \
    "Bob" \
    "$BOB_PRINCIPAL_ID" \
    "100_000_000"
  [ "$DEBUG" == 1 ] && echo $?
  
  allowancesForWICP \
    "$wicpId" \
    "$marketplaceId" \
    "50_000_000"
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 \
    "Alice" \
    "$ALICE_PRINCIPAL_ID" \
    "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForDIP721 \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection \
    "$DEFAULT_PRINCIPAL_ID" \
    "$marketplaceId" \
    "$nonFungibleContractAddress" \
    "$fungibleContractAddress" \
    "Crowns Collection" \
    10 \
    0
  [ "$DEBUG" == 1 ] && echo $?

  makeListing \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "1_250"
  [ "$DEBUG" == 1 ] && echo $?

  getAllListings "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  makeOffer \
    "$BOB_IDENTITY_NAME" \
    "Bob" \
    "$nonFungibleContractAddress" \
    "$nft_token_id_for_alice" \
    "1_300"
  [ "$DEBUG" == 1 ] && echo $?

  getAllOffers "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForNFTAcceptOffer \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "$wicpId"
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForWICPAcceptOffer \
    "$BOB_IDENTITY_NAME" \
    "$wicpId" \
    "$marketplaceId" \
    "5_000"
  [ "$DEBUG" == 1 ] && echo $?

  acceptOffer \
    "$ALICE_IDENTITY_NAME" \
    "$marketplaceId" \
    "$buy_offer_id"
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "👍 Healthcheck completed!"
