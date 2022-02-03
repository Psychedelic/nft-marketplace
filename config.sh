#!/bin/bash

export DFX_VERSION=0.8.1

DIP721Dir="/home/dan/dev/psy/tokens"
DIP20Dir="/home/dan/dev/psy/wicp"
MarketplaceDir="/home/dan/dev/psy/marketplace"

DIP721Script="${DIP721Dir}/nft/scripts/nft.sh"
DIP20Script="${DIP20Dir}/token.sh"
MarketplaceScript="${MarketplaceDir}/marketplace.sh"

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
    eval "(. ${MarketplaceScript}; $1)"
}