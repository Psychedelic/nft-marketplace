#!/bin/bash

# set -x

cd "$(dirname $BASH_SOURCE)" && cd ../../ || exit 1

# The NFT Canister id
nftCanisterId=$(cd crowns && dfx canister id crowns)
wicpCanisterId=$(cd  wicp && dfx canister id wicp)

token_index=0

# To use, provide the txt filename
# where each principal is in a new line
filename=$1

generateMock() {
  _identityName=$1
  _userPrincipal=$2

  printf "🤖 Call GenerateMock for identityName (%s), token_index (%s)" "$_identityName" "$token_index"

  crownsNftCanisterId="vlhm2-4iaaa-aaaam-qaatq-cai"
  filename=$(printf "%04d.mp4" "$token_index")
  crownsCertifiedAssetsA="vzb3d-qyaaa-aaaam-qaaqq-cai"
  crownsCertifiedAssetsB="vqcq7-gqaaa-aaaam-qaara-cai"
  assetUrl="https://$crownsCertifiedAssetsA.raw.ic0.app/$filename"

  dfx canister --network ic call $crownsNftCanisterId getMetadataDip721 "($token_index:nat64)"

  # Get some data from the mainnet canister
  mainnetMetadataResult=($(dfx canister --network ic call $crownsNftCanisterId getMetadataDip721 "($token_index:nat64)" | pcregrep -o1  '3_643_416_556 = "([a-zA-Z]*)"'))

  if [[ ! "$(declare -p mainnetMetadataResult)" =~ "declare -a" ]];
  then
    printf "👹 Oops! Metadata array is not fullfiled, will not proceed!"
    exit 1
  fi

  # Mint a token for the user
  # returns MintReceiptPart  { token_id: nat64; id: nat }
  printf "🤖 Mint NFT of id (%s) for user id (%s)\n\n" "$crownsNftCanisterId" "$_userPrincipal"

  # mint : (principal, nat, vec record { text; GenericValue }) -> (Result);
  mintResult=$(
    dfx canister --network local \
    call --update "$nftCanisterId" \
    mint "(
      principal \"$_userPrincipal\",
      $token_index:nat,
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
            \"TextContent\" = \"$assetUrl\"
          }
        };
      }
    )")

  printf "🤖 The mintResult is (%s)\n\n" "$mintResult"

  transactionId=$(echo "$mintResult" | pcregrep -o1 '17_724 = ([0-9]*)')

  printf "🤖 The generated transactionId is (%s)\n\n" "$transactionId"

  # # Show the metadata for the token
  printf "🤖 Call tokenMetadata for token id (%s)\n\n" "$token_index"
  dfx canister --network local \
    call "$nftCanisterId" tokenMetadata "($token_index:nat)"

  # # Show the owner of the token
  printf "🤖 Call tokenMetadata for token id (%s)\n\n" "$token_index"
  dfx canister --network local \
    call "$nftCanisterId" tokenMetadata "($token_index:nat)"

  # Increment token index
  token_index=$((token_index+1));
}

topupWicp() {
  _to_principal=$1
  _wicp_amount=$2

  printf "🤖 Topup (%s) WICP for user (%s)\n" "$_wicp_amount" "$_to_principal"

  dfx canister --network local \
    call --update "$wicpCanisterId" \
    transfer "( 
      principal \"$_to_principal\",
      $_wicp_amount:nat
    )"

  printf "🤖 The balanceOf user (%s) is\n" "$_to_principal"

  dfx canister --network local \
    call --update "$wicpCanisterId" \
    balanceOf "( 
      principal \"$_to_principal\"
    )"
}

generatorHandler() {
  _identityName=$1
  _userPrincipal=$2
  _total=$3

  # Iterator exec the mock generation incrementally
  for _ in $(seq 1 "$_total");
    do 
      printf "🤖 Will generate token mock for identity (%s), userPrincipal (%s), total (%s)\n\n" "$_identityName" "$_userPrincipal" "$_total"
      generateMock "$_identityName" "$_userPrincipal"
      topupWicp "$_userPrincipal" 1_000_000_000
  done
}

while read principal; do 
  generatorHandler "$principal" "$principal" 1
done < "$filename"


printf "✍️ Add collection to Marketplace\n"

./.scripts/add-collection.sh