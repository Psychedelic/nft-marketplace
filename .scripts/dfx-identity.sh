#!/bin/bash

initialIdentity=$(dfx identity whoami)
ALICE_IDENTITY_NAME="marketplace.alice"
BOB_IDENTITY_NAME="marketplace.bob"

dfx identity new "$ALICE_IDENTITY_NAME"
dfx identity use "$ALICE_IDENTITY_NAME" 

ALICE_PRINCIPAL_ID=$(dfx identity get-principal)
ALICE_WALLET=$(dfx identity get-wallet)

dfx identity new "$BOB_IDENTITY_NAME"
dfx identity use "$BOB_IDENTITY_NAME" 

BOB_PRINCIPAL_ID=$(dfx identity get-principal)
BOB_WALLET=$(dfx identity get-wallet)

dfx identity use "$initialIdentity" 

DEFAULT_PRINCIPAL_ID=$(dfx identity get-principal)
DEFAULT_USER_WALLET=$(dfx identity get-wallet)

dfx identity use "$initialIdentity"

printf "🙋‍♀️ Identities\n\n"

printf "👩🏽‍🦰 ALICE_PRINCIPAL_ID (%s)\n" "$ALICE_PRINCIPAL_ID"
printf "👩🏽‍🦰 ALICE_WALLET (%s)\n" "$ALICE_WALLET"

printf "👨🏽‍🦰 BOB_PRINCIPAL_ID (%s)\n" "$BOB_PRINCIPAL_ID"
printf "👨🏽‍🦰 BOB_WALLET (%s)\n" "$BOB_WALLET"

printf "👨🏾‍💻 DEFAULT_PRINCIPAL_ID (%s)\n" "$DEFAULT_PRINCIPAL_ID"

printf "🐷 DEFAULT_USER_WALLET (%s)\n" "$DEFAULT_USER_WALLET"

printf "\n\n"

DEFAULT_HOME=$HOME
ALICE_HOME=$HOME
BOB_HOME=$HOME