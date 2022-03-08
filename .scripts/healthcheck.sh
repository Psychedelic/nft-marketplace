#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x

. ".scripts/required/cap.sh"
. ".scripts/dfx-identity.sh"

marketplaceId=$(dfx canister id marketplace)
ownerPrincipalId=$DEFAULT_PRINCIPAL_ID
nonFungibleContractAddress=$(cd ./crowns && dfx canister id nft)
fungibleContractAddress=$(cd ./wicp && dfx canister id wicp)
icHistoryRouter=$(cd ./cap && dfx canister id ic-history-router)
wicpId="$(cd ./wicp && dfx canister id wicp)"
nft_token_id_for_alice=""
buy_offer_id=""

topupWICP() {
  printf "🤖 Call topupWICP\n"

  _callerHome=$1
  _wicpId=$2
  _name=$3
  _transferTo=$4
  _amount=$5

  printf "🤖 Will top-up (%s) account id (%s) by 
  transfer of amount (%s) \n" "$_name" "$_transferTo" "$_amount"

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --update "$_wicpId" \
    transfer "(
      principal \"$_transferTo\",
      $_amount:nat
    )"

  printf "🤖 balance of (%s)\n" "$_name"

  yarn wicp:balance-of "$_transferTo"
}

allowancesForWICP() {
  printf "🤖 Call allowancesForWICP\n"

  printf "🤖 Bob approves Marketplace (%s)\n" "$marketplaceId"

  _callerHome=$1
  _wicpId=$2
  _marketplaceId=$3
  _amount=$4

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --update "$_wicpId" \
    approve "(
      principal \"$_marketplaceId\",
      $_amount:nat
    )"
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
    HOME=$_callerHome \
    dfx canister --no-wallet \
      call --update "$_nonFungibleContractAddress" \
      mintDip721 "(
        principal \"$_mint_for\",
        vec {
          record {
            data = vec { (0:nat8) };
            key_val_data = vec {
              record {
                key= \"location\";
                val = variant {
                  TextContent = \"https://vqcq7-gqaaa-aaaam-qaara-cai.raw.ic0.app/0000.mp4\"
                }
              }
            };
            purpose = variant {
              Rendered
            };
          }
        }
      )"
    # TODO: While the ICX version works
    # Although the DFX version in the DIP721 works fine...
    # icx --pem="$DEFAULT_PEM" \
    # update "$_nonFungibleContractAddress" \
    # mintDip721 "(
    #   principal \"$_mint_for\",
    #   vec{}
    # )" \
    # --candid=./crowns/nft/candid/nft.did
  )

  nft_token_id_for_alice=$(echo "$_result" | pcregrep -o1  '726_683_809 = ([0-9]*)')

  printf "🤖 Minted Dip721 for user %s, has token ID (%s)\n" "$_name" "$nft_token_id_for_alice"

  printf "🤖 The Balance for user %s of id (%s)\n" "$_name" "$_mint_for"

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call "$_nonFungibleContractAddress" \
    balanceOfDip721 "(
      principal \"$_mint_for\",
      vec{}
    )"

  printf "🤖 User %s getMetadataForUserDip721 is\n" "$_name"

  HOME=$_callerHome \
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

allowancesForDIP721() {
  printf "🤖 Call the allowancesForDIP721\n"

  _callerHome=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _nft_token_id_for_alice=$4

  printf "🤖 Default approves Marketplace id (%s)\n 
  for non-fungible contract address (%s) 
  the token id (%s)" "$_marketplaceId"  "$_nonFungibleContractAddress" "$_nft_token_id_for_alice"

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call "$_nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat64
    )"
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

  # Interface
  # owner: Principal,
  # owner_fee_percentage: u16,
  # creation_time: u64,
  # collection_name: String,
  # non_fungible_contract_address: Principal,
  # non_fungible_token_type: NonFungibleTokenType,
  # fungible_contract_address: Principal,
  # fungible_token_type: FungibleTokenType,

  HOME=$_callerHome \
  dfx canister --no-wallet \
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

  _callerHome=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4
  _list_price=$5

  printf "🤖 has market id (%s)\n" "$_marketplaceId"
  printf "🤖 the token id is %s, price %s\n" "$_token_id" "$_list_price"

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --update "$_marketplaceId" \
    listForSale "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
        ($_list_price:nat)
      )"
}

getSaleOffers() {
    printf "🤖 Call getSaleOffers\n"

  _callerHome=$1
  _marketplaceId=$2

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --query "$_marketplaceId" \
    getSaleOffers "()"
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

  _result=$(
    HOME=$_callerHome \
    dfx canister --no-wallet \
      call --update "$_marketplaceId" \
      makeBuyOffer "(
        principal \"$_nonFungibleContractAddress\",
        $_token_id,
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

  _callerHome=$1
  _marketplaceId=$2
  _begin=$3
  _limit=$4

  printf "🤖 The getBuyOffers from marketplace id (%s) 
  was called with being (%s) and limit (%s)\n" "$_marketplaceId" "$_begin" "$_limit"

  HOME=$_callerHome \
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

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --update "$_nonFungibleContractAddress" \
    approveDip721 "(
      principal \"$_marketplaceId\",
      $_nft_token_id_for_alice:nat64
    )"
}

acceptBuyOffer() {
  printf "🤖 Call acceptBuyOffer\n"

  _callerHome=$1
  _marketplaceId=$2
  _buy_offer_id=$3

  HOME=$_callerHome \
  dfx canister --no-wallet \
    call --update "$_marketplaceId" \
    acceptBuyOffer "($_buy_offer_id:nat64)"
}

run() {
  printf "🚑 Healthcheck runtime details"
  printf "Owner address -> %s\n" "$ownerPrincipalId"

  topupWICP "$DEFAULT_HOME" "$wicpId" "Bob" "$BOB_PRINCIPAL_ID" "50_000_000"
  [ "$DEBUG" == 1 ] && echo $?
  
  allowancesForWICP "$BOB_HOME" "$wicpId" "$marketplaceId" "100_000_000"
  [ "$DEBUG" == 1 ] && echo $?

  mintDip721 "$DEFAULT_HOME" "Alice" "$ALICE_PRINCIPAL_ID" "$nonFungibleContractAddress"
  [ "$DEBUG" == 1 ] && echo $?

  allowancesForDIP721 "$DEFAULT_HOME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  addCrownCollection "$DEFAULT_HOME" \
    "$DEFAULT_PRINCIPAL_ID" \
    "$marketplaceId" \
    "$nonFungibleContractAddress" \
    "$fungibleContractAddress" \
    "Crowns Collection" \
    10 \
    0
  [ "$DEBUG" == 1 ] && echo $?

  listForSale "$ALICE_HOME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "1_250"
  [ "$DEBUG" == 1 ] && echo $?

  getSaleOffers "$HOME" "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  makeBuyOffer "$BOB_HOME" "$marketplaceId" "Bob" "$nft_token_id_for_alice" "1_000"
  [ "$DEBUG" == 1 ] && echo $?

  getBuyOffers "$HOME" "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveTransferFromForAcceptBuyOffer "$ALICE_HOME" "Alice" "$nonFungibleContractAddress" "$marketplaceId" "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  acceptBuyOffer "$ALICE_HOME" "$marketplaceId" "$buy_offer_id"
  [ "$DEBUG" == 1 ] &&  echo $?
}

run

echo "👍 Healthcheck completed!"