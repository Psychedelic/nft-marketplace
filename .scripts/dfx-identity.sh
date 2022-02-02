#!/bin/bash

ALICE_HOME=$(mktemp -d -t alice-temp)
BOB_HOME=$(mktemp -d -t bob-temp)

export ALICE_PRINCIPAL_ID=$(HOME=$ALICE_HOME dfx identity get-principal)
export BOB_PRINCIPAL_ID=$(HOME=$BOB_HOME dfx identity get-principal)
export DEFAULT_PRINCIPAL_ID=$(HOME=$HOME dfx identity get-principal)

export ALICE_PEM="$ALICE_HOME/.config/dfx/identity/default/identity.pem"
export BOB_PEM="$BOB_HOME/.config/dfx/identity/default/identity.pem"
export DEFAULT_PEM="$HOME/.config/dfx/identity/default/identity.pem"