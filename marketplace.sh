#!/bin/bash

dfxDir="/home/dan/.config/dfx"
candidDir="/home/dan/dev/psy/marketplace/marketplace"

MarketplaceCandidFile="${candidDir}/marketplace.did"
MarketplaceIcxPrologue="--candid=${MarketplaceCandidFile}"
MarketplaceId=$(dfx canister id marketplace)
NftContractId="qjdve-lqaaa-aaaaa-aaaeq-cai"

DefaultPem="${dfxDir}/identity/default/identity.pem"
AlicePem="${dfxDir}/identity/Alice/identity.pem"
BobPem="${dfxDir}/identity/Bob/identity.pem"
CharliePem="${dfxDir}/identity/Charlie/identity.pem"

DefaultPrincipalId=$(dfx identity use Default 2>/dev/null;dfx identity get-principal)
AlicePrincipalId=$(dfx identity use Alice 2>/dev/null;dfx identity get-principal)
BobPrincipalId=$(dfx identity use Bob 2>/dev/null;dfx identity get-principal)
CharliePrincipalId=$(dfx identity use Charlie 2>/dev/null;dfx identity get-principal)

dfx identity use default 2>/dev/null

nameToPrincipal=( ["Alice"]="$AlicePrincipalId" ["Bob"]="$BobPrincipalId" ["Charlie"]="$CharliePrincipalId" ["default"]="$DefaultPrincipalId")
nameToPem=( ["Alice"]="$AlicePem" ["Bob"]="$BobPem" ["Charlie"]="$CharliePem" ["Default"]="$DefaultPem")

help()
{
    printf "\n\nPrincipal ids\n"
    printf "Alice: ${AlicePrincipalId}\n"
    printf "Bob: ${BobPrincipalId}\n"
    printf "Charlie: ${CharliePrincipalId}\n"

    printf "\n\nAccount ids\n"
}

addCrownCollection() {
    ownerPrincipal="${nameToPrincipal[$1]}"
    nonFungibleContractAddress=$2
    fungibleContractAddress=$3
    icx --pem=$DefaultPem update $MarketplaceId addCollection "(principal \"$ownerPrincipal\", 10, 0, \"Crowns Collection\", principal \"$nonFungibleContractAddress\", variant { DIP721 }, principal \"$fungibleContractAddress\", variant { DIP20 })" $MarketplaceIcxPrologue
}

listForSale() {
    ownerPem="${nameToPem[$1]}"
    nonFungibleContractAddress=$2
    token_id=$3
    list_price=$4
    icx --pem=$ownerPem update $MarketplaceId listForSale "(principal \"$nonFungibleContractAddress\", $token_id, $list_price)" $MarketplaceIcxPrologue
}

getSaleOffers() {
    icx --pem=$DefaultPem query $MarketplaceId getSaleOffers "()" $MarketplaceIcxPrologue
}

makeBuyOffer() {
    ownerPem="${nameToPem[$1]}"
    nonFungibleContractAddress=$2
    token_id=$3
    list_price=$4
    icx --pem=$ownerPem update $MarketplaceId makeBuyOffer "(principal \"$nonFungibleContractAddress\", $token_id, $list_price)" $MarketplaceIcxPrologue
}

acceptBuyOffer() {
    ownerPem="${nameToPem[$1]}"
    buy_id=$2
    icx --pem=$ownerPem update $MarketplaceId acceptBuyOffer "($buy_id)" $MarketplaceIcxPrologue
}

getBuyOffers() {
    begin=$1
    limit=$2
    icx --pem=$DefaultPem query $MarketplaceId getBuyOffers "($begin, $limit)" $MarketplaceIcxPrologue
}
