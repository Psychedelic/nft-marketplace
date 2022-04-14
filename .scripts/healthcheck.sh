#!/bin/bash

# set -x

(cd "$(dirname $BASH_SOURCE)" && cd ..) || exit 1

[ "$DEBUG" == 1 ] && set -x


. ".scripts/required/cap.sh"
. ".scripts/dfx-identity.sh"

set -e

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

  crownsNftCanisterId="vlhm2-4iaaa-aaaam-qaatq-cai"

  printf "🤖 The mintDip721 has nonFungibleContractAddress (%s), 
  mint_for user (%s) of id (%s)\n" "$_nonFungibleContractAddress" "$_name" "$_mint_for"

  example_nft=$(echo "$RANDOM%5000 + 1" | bc)
  mainnetMetadataResult=($(dfx canister --network ic call $crownsNftCanisterId getMetadataDip721 "($example_nft:nat64)" | pcregrep -o1  '3_643_416_556 = "([a-zA-Z]*)"'))

  echo $mainnetMetadataResult

  nft_token_id_for_alice=$(echo "$example_nft + 10000" | bc) # shift out of crowns index space

  _result=$(
    dfx canister \
      call --update "$_nonFungibleContractAddress" \
      mint "(
        principal \"$_mint_for\",
        $nft_token_id_for_alice:nat,
        vec {
          record {
            \"smallgem\";
            variant {
              \"TextContent\" = \"${mainnetMetadataResult[0]}\"
            }
          };
          record {
            \"biggem\";
            variant {
              \"TextContent\" = \"${mainnetMetadataResult[1]}\"
            }
          };
          record {
            \"base\";
            variant {
              \"TextContent\" = \"${mainnetMetadataResult[2]}\"
            }
          };
          record {
            \"rim\";
            variant {
              \"TextContent\" = \"${mainnetMetadataResult[3]}\"
            }
          };
          record {
            \"location\";
            variant {
              \"TextContent\" = \"https://vzb3d-qyaaa-aaaam-qaaqq-cai.raw.ic0.app/$example_nft.mp4\"
            }
          };
          record {
            \"thumbnail\";
            variant {
              \"TextContent\" = \"https://vzb3d-qyaaa-aaaam-qaaqq-cai.raw.ic0.app/thumbnail/$example_nft.png\"
            }
          };
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
  # owner_fee_percentage: Nat,
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
        ($_fee:nat),
        ($_creationTime:nat64),
        \"$_collectionName\",
        principal \"$_nonFungibleContractAddress\",
        variant { DIP721v2 },
        principal \"$_fungibleContractAddress\",
        variant { DIP20 }
      )"
}

depositFungible() {
  echo "🤖 Deposit Fungible"
  
  _identityName=$1
  _FungibleContractAddress=$2
  _marketplaceId=$3
  _amount=$4

  dfx --identity "$_identityName" \
    canister call \
    --update $_marketplaceId depositFungible \
    "(
      principal \"$_FungibleContractAddress\",
      variant { DIP20 },
      $_amount:nat
    )"
}

withdrawFungible() {
  echo "🤖 Withdraw Fungible"

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4

  dfx --identity "$_identityName" \
    canister call \
    --update $_marketplaceId withdrawFungible \
    "(
      principal \"$_nonFungibleContractAddress\",
      variant { DIP20 }
    )"
}

depositNFT() {
  echo "🤖 Deposit NFT"
  
  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4

  dfx --identity "$_identityName" \
    canister call \
    --update $_marketplaceId depositNFT \
    "(
      principal \"$_nonFungibleContractAddress\",
      $_token_id:nat64
    )"
}

withdrawNFT() {
  echo "🤖 Withdraw NFT"

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4

  dfx --identity "$_identityName" \
    canister call \
    --update $_marketplaceId withdrawNFT \
    "(
      principal \"$_nonFungibleContractAddress\",
      $_token_id:nat64
    )"
}

getAllBalances() {
    printf "🤖 Call getAllBalances (fungibles)\n"

  _marketplaceId=$1

  dfx canister \
    call --query "$_marketplaceId" \
    getAllBalances "()"
}

makeListing() {
  printf "🤖 Call makeListing\n"

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4
  _list_price=$5
  _direct_buy=$6

  printf "🤖 has market id (%s)\n" "$_marketplaceId"
  printf "🤖 the token id is %s, price %s, and direct buy %s\n" "$_token_id" "$_list_price" "$_direct_buy"
  printf "🤖 will use identity %s\n" "$_identityName"

  dfx --identity "$_identityName" \
    canister call --update "$_marketplaceId" \
    makeListing "(
        $_direct_buy:bool,
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

  dfx --identity "$_identityName" \
    canister call --update "$_marketplaceId" \
    makeOffer "(
      principal \"$_nonFungibleContractAddress\",
      ($_token_id:nat64),
      ($_offer_price:nat)
    )"
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

approveNFT() {
  printf "🤖 Call approveNFT\n"  

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _nft_token_id_for_alice=$4

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


approveFungible() {
  printf "🤖 Call approveFungible\n"

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
  _nonFungibleContractAddress=$3
  _token_id=$4
  _buyerId=$5

  dfx --identity "$_identityName" \
    canister \
    call --update "$_marketplaceId" \
    acceptOffer "(
      principal \"$_nonFungibleContractAddress\",
      $_token_id:nat64,
      principal \"$_buyerId\"
    )"
}

directBuy() {
  printf "🤖 Call directBuy\n"

  _identityName=$1
  _nonFungibleContractAddress=$2
  _marketplaceId=$3
  _token_id=$4

  dfx --identity "$_identityName" \
    canister call \
    --update $_marketplaceId directBuy \
    "(
      principal \"$nonFungibleContractAddress\",
      $_token_id:nat64
    )"
}

accept_offer() {
  printf "\n🌊 Make/Accept Offer Flow\n\n"

  makeListing \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "1_200" \
    "false"
  [ "$DEBUG" == 1 ] && echo $?

  getAllListings "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  approveFungible \
    "$BOB_IDENTITY_NAME" \
    "$wicpId" \
    "$marketplaceId" \
    "600"
  [ "$DEBUG" == 1 ] && echo $?

  makeOffer \
    "$BOB_IDENTITY_NAME" \
    "Bob" \
    "$nonFungibleContractAddress" \
    "$nft_token_id_for_alice" \
    "600"
  [ "$DEBUG" == 1 ] && echo $?

  getAllOffers "$marketplaceId" 0 10
  [ "$DEBUG" == 1 ] && echo $?

  approveNFT \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  depositNFT \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" 
  [ "$DEBUG" == 1 ] && echo $?

  echo "halting crowns cansiter to test balance fallback..."

  $(cd crowns && dfx canister stop crowns)

  acceptOffer \
    "$ALICE_IDENTITY_NAME" \
    "$marketplaceId" \
    "$nonFungibleContractAddress" \
    "$nft_token_id_for_alice" \
    "$BOB_PRINCIPAL_ID"
  [ "$DEBUG" == 1 ] && echo $?

  echo "balance of bob"

  dfx canister call marketplace serviceBalanceOf "(principal \"$(dfx --identity $BOB_IDENTITY_NAME identity get-principal)\")"

  echo "balance of alice"

  dfx canister call marketplace serviceBalanceOf "(principal \"$(dfx --identity $ALICE_IDENTITY_NAME identity get-principal)\")"

  echo "starting crowns canister to test withdraw from balance..."

  $(cd crowns && dfx canister start crowns)

  withdrawNFT \
    "$BOB_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" 

  getAllBalances "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  return 0
}


direct_buy() {
  printf "\n🌊 Direct Buy Flow\n\n"

  approveNFT \
    "$BOB_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice"
  [ "$DEBUG" == 1 ] && echo $?

  depositNFT \
    "$BOB_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" 
  [ "$DEBUG" == 1 ] && echo $?
  
  makeListing \
    "$BOB_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" \
    "500" \
    "true"
  [ "$DEBUG" == 1 ] && echo $?

  getAllListings "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  approveFungible \
    "$ALICE_IDENTITY_NAME" \
    "$wicpId" \
    "$marketplaceId" \
    "500"
  [ "$DEBUG" == 1 ] && echo $?

  depositFungible \
    "$ALICE_IDENTITY_NAME" \
    "$wicpId" \
    "$marketplaceId" \
    "500" 
  [ "$DEBUG" == 1 ] && echo $?

  echo "halting wicp canister to test balance fallback ..."
  $(cd wicp && dfx canister stop wicp)

  directBuy \
    "$ALICE_IDENTITY_NAME" \
    "$nonFungibleContractAddress" \
    "$marketplaceId" \
    "$nft_token_id_for_alice" 
  [ "$DEBUG" == 1 ] && echo $?

  getAllBalances "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  echo "starting wicp canister to test withdraw..."
  $(cd wicp && dfx canister start wicp)

  withdrawFungible \
    "$BOB_IDENTITY_NAME" \
    "$wicpId" \
    "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  getAllBalances "$marketplaceId"
  [ "$DEBUG" == 1 ] && echo $?

  return 0
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

  accept_offer
  direct_buy

  echo "👍 Healthcheck completed!"

  return $?
}

run && exit 0
