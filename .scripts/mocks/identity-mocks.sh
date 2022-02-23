#!/bin/bash

cd "$(dirname $BASH_SOURCE)" && cd ../../ && mkdir -p .mocks && cd .mocks || exit 1

aliceHome=$(mkdir -p alice && echo "$(pwd)/alice")
bobHome=$(mkdir -p bob && echo "$(pwd)/bob")

alicePrincipalId=$(HOME=$aliceHome dfx identity get-principal)
bobPrincipalId=$(HOME=$bobHome dfx identity get-principal)
dfxUserPrincipalId=$(dfx identity get-principal)

printf "🙋‍♀️ Identities\n\n"

printf "👩🏽‍🦰 Alice principal id (%s)\n" "$alicePrincipalId"
printf "👩🏽‍🦰 Alice home (%s)\n" "$aliceHome"

printf "👨🏽‍🦰 Bob principal id (%s)\n" "$bobPrincipalId"
printf "👨🏽‍🦰 Bob home (%s)\n" "$bobHome"

printf "👨🏾‍💻 Dfx user principal id (%s)\n" "$dfxUserPrincipalId"
printf "👨🏾‍💻 Dfx user home (%s)\n" "$HOME"

printf "\n\n"