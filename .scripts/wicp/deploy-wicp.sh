#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../wicp || exit 1

_wallet="$1"
_ic_history_router="$2"
_amount="$3"

dfx deploy \
wicp --argument="(
				\"data:image/jpeg;base64,$(base64 ../.repo/images/logo-of-wicp.png)\",
				\"wicp\",
				\"WICP\",
				8:nat8,
				$_amount:nat,
				principal \"$_wallet\", 
				0, 
				principal \"$_wallet\", 
				principal \"$_ic_history_router\"
				)" 