#!/bin/bash

# set -x

(cd "$(dirname $BASH_SOURCE)" && cd ../../) || exit 1

# . ".scripts/mocks/identity-mocks.sh"
. ".scripts/dfx-identity.sh"

# The NFT Canister id
nftCanisterId=$1

# The total tokens to generate
totalNumberOfTokens=$2

token_index=$3

if [[ -z $3 ]];
then
  printf "🤖 The token index start from not provided (default is 0)\n"
  token_index=0
fi

generateMock() {
  _identityName=$1
  _userPrincipal=$2

  printf "🤖 Call GenerateMock for identityName (%s), token_index (%s)" "$_identityName" "$token_index"

  crownsNftCanisterId="vlhm2-4iaaa-aaaam-qaatq-cai"
  filename=$(printf "%04d.mp4" "$token_index")
  crownsCertifiedAssetsA="vzb3d-qyaaa-aaaam-qaaqq-ca"
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

userIdentityWarning() {
  _identity=$1

  # The extra white space is intentional, used for alignment
  read -r -p "⚠️  Is your dfx identity Plug's (exported PEM) [Y/n]? " CONT

  if [ "$CONT" = "Y" ]; then
    printf "🌈 The DFX Identity is set to (%s), make sure it matches Plug's!\n\n" "$_identity"
  else
    printf "🚩 Make sure you configure DFX cli to use Plug's exported identity (PEM) \n\n"

    exit 1;
  fi
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
  done
}

# Distribute the total number of tokens
dividedTotal=$((totalNumberOfTokens / 3))
printf "$dividedTotal"
dividedTotal=$(echo "$dividedTotal" | awk '{print int($1+0.5)}')

userTotal=$((dividedTotal + (totalNumberOfTokens - ($dividedTotal*3))))

# Warn the user about identity requirement
# as the end user will be interacting with the Marketplace via Plug's
userIdentityWarning "$DEFAULT_PRINCIPAL_ID"

# generates mock data for the dfx user principal
generatorHandler "$INITIAL_IDENTITY" "$DEFAULT_PRINCIPAL_ID" "$userTotal"

# generates mock data for Alice
generatorHandler "$ALICE_IDENTITY_NAME" "$ALICE_PRINCIPAL_ID" "$dividedTotal"

# generates mock data for Bob
generatorHandler "$BOB_IDENTITY_NAME" "$BOB_PRINCIPAL_ID" "$dividedTotal"

printf "💡 Use the identities in DFX Cli by providing it via flag --identity\n"
printf "this is useful because you can interact with the Marketplace with different identities\n"
printf "to create the necessary use-case scenarios throughout development\n"
printf "\n"
printf "👩🏽‍🦰 Alice identity name (%s)\n" "$ALICE_IDENTITY_NAME"
printf "👨🏽‍🦰 Bob identity name (%s)\n" "$BOB_IDENTITY_NAME"
