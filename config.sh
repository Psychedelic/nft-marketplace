#!/usr/bin/bash
export DFX_VERSION=0.8.1

CAPDir="/home/dan/dev/psy/cap"
DIP721Dir="/home/dan/dev/psy/tokens"
DIP20Dir="/home/dan/dev/psy/wicp"
DabDir="/home/dan/dev/psy/dab"
ExtDir="/home/dan/dev/extendable-token"
MarketplaceDir="/home/dan/dev/psy/marketplace"

DIP721Script="${DIP721Dir}/nft/scripts/nft.sh"
DIP20Script="${DIP20Dir}/token.sh"
MarketplaceScript="${MarketplaceDir}/marketplace.sh"

FinishedSound="/home/dan/dev/sounds/finished.mp3"

DefaultPrincipalId=$(dfx identity use Default 2>/dev/null;dfx identity get-principal)
AlicePrincipalId=$(dfx identity use Alice 2>/dev/null;dfx identity get-principal)
BobPrincipalId=$(dfx identity use Bob 2>/dev/null;dfx identity get-principal)
CharliePrincipalId=$(dfx identity use Charlie 2>/dev/null;dfx identity get-principal)

dfx identity use default

DIP721Exec() {
    eval "(. ${DIP721Script} dip721mp; $1)"
}

DIP20Exec() {
    echo $1
    eval "(. ${DIP20Script} dip20mp; $1)"
}

MarketplaceExec() {
    echo $1
    eval "(. ${MarpetplaceScript}; $1)"
}
