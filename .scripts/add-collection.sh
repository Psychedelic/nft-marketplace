#!/bin/bash

printf "🤖 Add collection to the Marketplace\n"

cd "$(dirname $BASH_SOURCE)" && cd .. || exit 1

(
  ownerPrincipalId=$(dfx identity get-principal)
  marketplaceId=$(dfx canister id marketplace)
  fee=1
  creationTime=0
  collectionName="Crowns"
  nonFungibleContractAddress=$(cd crowns && dfx canister id crowns)
  fungibleContractAddress=$(cd wicp && dfx canister id wicp)

  echo "ownerPrincipalId > $ownerPrincipalId"
  echo "marketplaceId > $marketplaceId"
  echo "fee > $fee"
  echo "creationTime > $creationTime"
  echo "nonFungibleContractAddress > $nonFungibleContractAddress"
  echo "fungibleContractAddress > $fungibleContractAddress"

  dfx canister \
    call --update "$marketplaceId" \
    addCollection "(
        principal \"$ownerPrincipalId\",
        ($fee:nat),
        ($creationTime:nat64),
        \"$collectionName\",
        principal \"$nonFungibleContractAddress\",
        variant { DIP721v2 },
        principal \"$fungibleContractAddress\",
        variant { DIP20 }
      )"
)
