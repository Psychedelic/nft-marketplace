#!/bin/bash

# set -x

cd "$(dirname $BASH_SOURCE)" && cd ../../ || exit 1

. ".scripts/dfx-identity.sh"

# The NFT Canister id
nftCanisterId=$(cd crowns && dfx canister id crowns)
wicpCanisterId=$(cd  wicp && dfx canister id wicp)

# The total tokens to generate
totalNumberOfTokens=$1

token_index=$2

if [[ -z $token_index ]];
then
  printf "🤖 The token index start from not provided (default is 0)\n"
  token_index=0
fi

# The location for the Crowns aggregated data (backup on v2 format)
aggrCrownsDataPath=./crowns/migrate/03_aggregate.json
crownsNftCanisterId="vlhm2-4iaaa-aaaam-qaatq-cai"

generateMock() {
  _identityName=$1
  _userPrincipal=$2

  printf "🤖 Call GenerateMock for identityName (%s), token_index (%s)\n\n" "$_identityName" "$token_index"

  smallgem=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[0][1] | .TextContent")
  biggem=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[1][1] | .TextContent")
  base=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[2][1] | .TextContent")
  rim=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[3][1] | .TextContent")
  location=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[4][1] | .TextContent")
  thumbnail=$(cat $aggrCrownsDataPath | jq ".[$token_index] | .properties[5][1] | .TextContent")

  printf "smallgem (%s), biggem (%s), base (%s), rim (%s), location (%s), thumbnail (%s)\n\n" "$smallgem" "$biggem" "$base" "$rim" "$location" "$thumbnail"

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
            \"TextContent\" = $smallgem
          }
        };
        record {
          \"biggem\";
          variant {
            \"TextContent\" = $biggem
          }
        };
        record {
          \"base\";
          variant {
            \"TextContent\" = $base
          }
        };
        record {
          \"rim\";
          variant {
            \"TextContent\" = $rim
          }
        };
        record {
          \"location\";
          variant {
            \"TextContent\" = $location
          }
        };
        record {
          \"thumbnail\";
          variant {
            \"TextContent\" = $thumbnail
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

# Distribute the total number of tokens
dividedTotal=$((totalNumberOfTokens / 3))
dividedTotal=$(echo "$dividedTotal" | awk '{print int($1+0.5)}')

userTotal=$((dividedTotal + (totalNumberOfTokens - (dividedTotal*3))))

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

printf "✍️ Add collection to Marketplace\n"

./.scripts/add-collection.sh