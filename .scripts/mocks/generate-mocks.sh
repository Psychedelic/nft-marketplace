#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ../../) || exit 1

# The NFT Canister id
nftCanisterId=$1

# Which user principal to mint for
mintForPrincipalId=$2

# The total tokens to generate
totalNumberOfTokens=$3

generateMock() {
  token_index=$1

  crownsNftCanisterId="vlhm2-4iaaa-aaaam-qaatq-cai"
  filename=$(printf "%04d.mp4" "$token_index")
  assetUrl="https://$crownsNftCanisterId.raw.ic0.app/$filename"

  # Get some data from the mainnet canister
  mainnetMetadataResult=($(dfx canister --network ic call $crownsNftCanisterId getMetadataDip721 "($token_index:nat64)" | pcregrep -o1  '3_643_416_556 = "([a-zA-Z]*)"'))

  if [[ ! "$(declare -p mainnetMetadataResult)" =~ "declare -a" ]];
  then
    printf "👹 Oops! Metadata array is not fullfiled, will not proceed!"
    exit 1
  fi

  # Mint a token for the user
  # returns MintReceiptPart  { token_id: nat64; id: nat }
  printf "🤖 Mint NFT of id (%s) for user id (%s)\n\n" "$nftCanisterId" "$mintForPrincipalId"

  mintResult=$(dfx canister --no-wallet \
    --network local \
    call --update "$nftCanisterId" \
    mintDip721 "(
      principal \"$mintForPrincipalId\",
      vec {
        record {
          data = vec { (0:nat8) };
          key_val_data = vec {
            record {
              key = \"smallgem\";
              val = variant {
                TextContent = \"${mainnetMetadataResult[0]}\"
              };
            };
            record {
              key = \"biggem\";
              val = variant {
                TextContent = \"${mainnetMetadataResult[1]}\"
              };
            };
            record {
              key = \"base\";
              val = variant {
                TextContent = \"${mainnetMetadataResult[2]}\"
              };
            };
            record {
              key = \"rim\";
              val = variant {
                TextContent = \"${mainnetMetadataResult[3]}\"
              };
            };
            record {
              key= \"location\";
              val = variant {
                TextContent = \"$assetUrl\"
              }
            }
          };
          purpose = variant {
            Rendered
          };
        }
      }
    )")

  mintTokenId=$(echo "$mintResult" | pcregrep -o1  '= ([0-9]*) : nat64')

  printf "🤖 The generated token id (%s)\n\n" "$mintTokenId"

  # # Show the metadata for the token
  printf "🤖 Call getMetadataDip721 for token id (%s)\n\n" "$mintTokenId"
  dfx canister --network local \
    call "$nftCanisterId" getMetadataDip721 "($mintTokenId:nat64)"

  # # Show the owner of the token
  printf "🤖 Call ownerOfDip721 for token id (%s)\n\n" "$mintTokenId"
  dfx canister --network local \
    call "$nftCanisterId" ownerOfDip721 "($mintTokenId:nat64)"

}

userIdentityWarning() {
  # The extra white space is intentional, used for alignment
  read -r -p "⚠️  Are you are aware that the dfx identity should be Plug's (SECP256K1) [Y/n]? " CONT

  if [ "$CONT" = "Y" ]; then
    printf "🌈 The DFX Identity is set to (%s)\n\n" "$(dfx identity get-principal), make sure it matches Plug's"
  else
    printf "🚩 Make sure you configure DFX cli to use Plug's exported identity (PEM) \n\n"

    exit 1;
  fi
}

# Warn the user about identity requirement
# as the end user will be interacting with the Marketplace via Plug's
userIdentityWarning

# Iterator exec the mock generation incrementally
for i in $(seq 1 "$totalNumberOfTokens");
  do generateMock "$i"
done
