#!/bin/bash

# The NFT Canister id
nftCanisterId=$1

# Which user principal to mint for
mintForPrincipalId=$2

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
              TextContent = \"Pearl\"
            };
          };
          record {
            key = \"biggem\";
            val = variant {
              TextContent = \"GreenOrb\"
            };
          };
          record {
            key = \"base\";
            val = variant {
              TextContent = \"Silver\"
            };
          };
          record {
            key = \"rim\";
            val = variant {
              TextContent = \"Engraved\"
            };
          };
          record {
            key= \"location\";
            val = variant {
              TextContent = \"https://vzb3d-qyaaa-aaaam-qaaqq-cai.raw.ic0.app/0000.mp4\"
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
