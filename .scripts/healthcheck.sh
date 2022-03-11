#!/bin/bash

# set -x

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

. ".scripts/required/cap.sh"
. ".scripts/dfx-identity.sh"

marketplaceId=$(dfx canister id marketplace)
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=$(cd ./crowns && dfx canister id crowns)
fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)
icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)
wicpId="$(cd ./wicp && dfx canister id wicp)"
nft_token_id_for_alice=""
buy_offer_id=""

topupWICP() {
  printf "🤖 Call topupWICP\n"

  _wallet=$1
  _wicpId=$2
  _name=$3
  _transferTo=$4
  _amount=$5

  printf "🤖 Will top-up (%s) account id (%s) by 
  transfer of amount (%s) \n" "$_name" "$_transferTo" "$_amount"

  dfx canister --wallet "$_wallet" \
    call --update "$_wicpId" \
    transfer "(
      principal \"$_transferTo\",
      $_amount:nat
    )"

  printf "🤖 balance of (%s)\n" "$_name"

  # TODO: wicp:balance-of refactor or remove
  # yarn wicp:balance-of "$_transferTo"

  dfx canister --wallet "$_wallet" \
    call --query "$_wicpId" \
    balanceOf "(
      principal \"$_transferTo\"
    )"
}

allowancesForWICP() {
  printf "🤖 Call allowancesForWICP\n"

  printf "🤖 Bob approves Marketplace (%s)\n" "$marketplaceId"

  _wallet=$1
  _wicpId=$2
  _marketplaceId=$3
  _amount=$4

  dfx canister --wallet "$_wallet" \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"
}

mintDip721() {
  printf "🤖 Call the mintDip721\n"

  _wallet=$1
  _name=$2
  _mint_for=$3
  _nonFungibleContractAddress=$4

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), 
  mint_for user (%s) of id (%s)\n" "$_nonFungibleContractAddress" "$_name" "$_mint_for"

  nft_token_id_for_alice=$RANDOM

  _result=$(
    dfx canister --wallet "$_wallet" \
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

  dfx canister --wallet "$_wallet" \
    call "$_nonFungibleContractAddress" \
    balanceOf "(
      principal \"$_mint_for\",
    )"

  printf "🤖 User %s ownerTokenMetadata is\n" "$_name"

  dfx canister --wallet "$_wallet" \
    call "$_nonFungibleContractAddress" \
    ownerTokenMetadata "(
      principal \"$_mint_for\",
    )"

  printf "\n"
}

allowancesForDIP721() {
  printf "🤖 Call the allowancesForDIP721\n"

  _wallet=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _nft_token_id_for_alice=$4

  printf "🤖 Default approves Marketplace id (%s)\n 
  for non-fungible contract address (%s) 
  the token id (%s)" "$_marketplaceId"  "$_nonFungibleContractAddress" "$_nft_token_id_for_alice"

  dfx canister --wallet "$_wallet" \
    call "$_nonFungibleContractAddress" \
    approve "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat
    )"
}

addCrownCollection() {
  echo "🤖 Add Crown Collection"

  _wallet=$1
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

  # Interface
  # owner: Principal,
  # owner_fee_percentage: u16,
  # creation_time: u64,
  # collection_name: String,
  # non_fungible_contract_address: Principal,
  # non_fungible_token_type: NonFungibleTokenType,
  # fungible_contract_address: Principal,
  # fungible_token_type: FungibleTokenType,

  dfx canister --wallet "$_wallet" \
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

listForSale() {
  printf "🤖 List for sale\n"

  _wallet=$1
  _identityName=$2
  _nonFungibleContractAddress=$3
  _marketplaceId=$4
  _token_id=$5
  _list_price=$6

  printf "🤖 has market id (%s)\n" "$_marketplaceId"
  printf "🤖 the token id is %s, price %s\n" "$_token_id" "$_list_price"
  printf "🤖 will use identity %s\n" "$_identityName"

  dfx --identity "$_identityName" \
    canister --wallet "$_wallet" \
    call --update "$_marketplaceId" \
    listForSale "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
        ($_list_price:nat)
      )"
}

getSaleOffers() {
    printf "🤖 Call getSaleOffers\n"

  _wallet=$1
  _marketplaceId=$2

  dfx canister --wallet "$_wallet" \
    call --query "$_marketplaceId" \
    getSaleOffers "()"
}

makeBuyOffer() {
  printf "🤖 Call makeBuyOffer\n"

  _wallet=$1
  _identityName=$2
  _name=$3
  _nonFungibleContractAddress=$4
  _token_id=$5
  _offer_price=$6

  printf "🤖 The user (%s) will makeBuyOffer for token id (%s) 
  for the amount (%s)\n" "$_name" "$_token_id" "$_offer_price"

  # TODO: re-enable the log for balance of
  # printf "🤖 balance of (%s) is equal to\n" "$_name"
  # yarn wicp:balance-of "$_transferTo"

  _result=$(
    dfx --identity "$_identityName" \
      canister --wallet "$_wallet" \
      call --update "$_marketplaceId" \
      makeBuyOffer "(
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

getBuyOffers() {
  printf "🤖 Call getBuyOffers\n"

  _wallet=$1
  _marketplaceId=$2
  _begin=$3
  _limit=$4

  printf "🤖 The getBuyOffers from marketplace id (%s) 
  was called with being (%s) and limit (%s)\n" "$_marketplaceId" "$_begin" "$_limit"

  dfx canister --wallet "$_wallet" \
    call --query "$_marketplaceId" \
    getBuyOffers "($_begin, $_limit)"
}

approveTransferFromForNFTAcceptBuyOffer() {
  printf "🤖 Call approveTransferFromForNFTAcceptBuyOffer\n"  

  _wallet=$1
  _identityName=$2
  _name=$3
  _approves_wallet=$4
  _nonFungibleContractAddress=$5
  _marketplaceId=$6
  _nft_token_id_for_alice=$7
  _wicpId=$8

  printf "🤖 The user (%s) will approve transfer token id (%s) 
  for user (%s) \n" "$_name" "$_nft_token_id_for_alice" "$_approves_wallet"
  printf "🤖 for nft contract id (%s)\n" "$_nonFungibleContractAddress"

  dfx --identity "$_identityName" \
    canister --wallet "$_wallet" \
    call --update "$_nonFungibleContractAddress" \
    approve "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat
    )"
}


approveTransferFromForWICPAcceptBuyOffer() {
  printf "🤖 Call approveTransferFromForWICPAcceptBuyOffer\n"

  _wallet=$1
  _identityName=$2
  _wicpId=$3
  _marketplaceId=$4
  _amount=$5

  printf "🤖 The user (%s) approves (%s) for the amount (%s) in fungible token (%s)" "$_identityName" "$_marketplaceId" "$_amount" "$_wicpId"

  dfx --identity "$_identityName" \
    canister --wallet "$_wallet" \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"
}

acceptBuyOffer() {
  printf "🤖 Call acceptBuyOffer\n"

  _wallet=$1
  _identityName=$2
  _marketplaceId=$3
  _buy_offer_id=$4

  dfx --identity "$_identityName" \
    canister --wallet "$_wallet" \
    call --update "$_marketplaceId" \
    acceptBuyOffer "($_buy_offer_id:nat64)"
}

run() {
  printf "🚑 Healthcheck runtime details\n"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  topupWICP "$DEFAULT_USER_WALLET" "$wicpId" "Bob" "$BOB_WALLET" "100_000_000"
  [ "$DEBUG" == 1 ] && echo $?
  
  allowancesForWICP "$DEFAULT_USER_WALLET" "$wicpId" "$marketplaceId" "50_000_000"
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 "$DEFAULT_USER_WALLET" "Alice" "$ALICE_WALLET" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForDIP721 "$DEFAULT_USER_WALLET" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection "$DEFAULT_USER_WALLET" \
    "$DEFAULT_PRINCIPAL_ID" \
    "$marketplaceId" \
    "$nonFungibleContractAddress" \
    "$fungibleContractAddress" \
    "Crowns Collection" \
    10 \
    0
  [ "$DEBUG" == 1 ] && echo $?

  listForSale "$ALICE_WALLET" \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "1_250"
  [ "$DEBUG" == 1 ] && echo $?

  getSaleOffers "$DEFAULT_USER_WALLET" "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  makeBuyOffer "$BOB_WALLET" \
    "$BOB_IDENTITY_NAME" \
    "Bob" \
    "$nonFungibleContractAddress" \
    "$nft_token_id_for_alice" \
    "1_300"
  [ "$DEBUG" == 1 ] && echo $?

  getBuyOffers "$DEFAULT_USER_WALLET" "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForNFTAcceptBuyOffer "$ALICE_WALLET" "$ALICE_IDENTITY_NAME" "Alice" "$BOB_WALLET" "$nonFungibleContractAddress" "$marketplaceId" "$nft_token_id_for_alice" "$wicpId"
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForWICPAcceptBuyOffer "$BOB_WALLET" "$BOB_IDENTITY_NAME" "$wicpId" "$marketplaceId" "5_000"
  [ "$DEBUG" == 1 ] && echo $?

  acceptBuyOffer "$ALICE_WALLET" "$ALICE_IDENTITY_NAME" "$marketplaceId" "$buy_offer_id"
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "🤖 Clear identities for Alice and Bob"

dfx identity remove "$ALICE_IDENTITY_NAME"
dfx identity remove "$BOB_IDENTITY_NAME"

echo "👍 Healthcheck completed!"