#!/bin/bash

# ALICE_HOME=$(mktemp -d 2>/dev/null || mktemp -d -t alice-temp)
# BOB_HOME=$(mktemp -d 2>/dev/null || mktemp -d -t bob-temp)
ALICE_HOME=$(mkdir -p ./.dfx/identities/alice)
BOB_HOME=$(mkdir -p ./.dfx/identities/bob)
DEFAULT_HOME="$HOME"

ALICE_PRINCIPAL_ID=$(HOME=$ALICE_HOME dfx identity get-principal)
BOB_PRINCIPAL_ID=$(HOME=$BOB_HOME dfx identity get-principal)
DEFAULT_PRINCIPAL_ID=$(HOME=$HOME dfx identity get-principal)

# TODO: Investigate why the wallet addresses are the same?
ALICE_WALLET=$(HOME=$ALICE_HOME dfx identity get-wallet)
BOB_WALLET=$(HOME=$BOB_HOME dfx identity get-wallet)

ALICE_PEM="$ALICE_HOME/.config/dfx/identity/default/identity.pem"
BOB_PEM="$BOB_HOME/.config/dfx/identity/default/identity.pem"
DEFAULT_PEM="$HOME/.config/dfx/identity/default/identity.pem"

DEFAULT_USER_WALLET=$(dfx identity get-wallet)

printf "🙋‍♀️ Identities\n\n"

printf "👩🏽‍🦰 ALICE_PRINCIPAL_ID (%s)\n" "$ALICE_PRINCIPAL_ID"
printf "👩🏽‍🦰 ALICE_WALLET (%s)\n" "$ALICE_WALLET"
printf "👩🏽‍🦰 ALICE_HOME (%s)\n" "$ALICE_HOME"

printf "👨🏽‍🦰 BOB_PRINCIPAL_ID (%s)\n" "$BOB_PRINCIPAL_ID"
printf "👨🏽‍🦰 BOB_WALLET (%s)\n" "$BOB_WALLET"
printf "👨🏽‍🦰 BOB_HOME (%s)\n" "$BOB_HOME"

printf "👨🏾‍💻 DEFAULT_PRINCIPAL_ID (%s)\n" "$DEFAULT_PRINCIPAL_ID"
printf "👨🏾‍💻 DEFAULT_HOME (%s)\n" "$DEFAULT_HOME"

printf "🐷 DEFAULT_USER_WALLET (%s)\n" "$DEFAULT_USER_WALLET"

printf "\n\n"