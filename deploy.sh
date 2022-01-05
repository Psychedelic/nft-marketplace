#!/usr/bin/bash
. ./config.sh

if [[ "$RESTARTIC" ]]
then
    ################## CLEANUP ##################

    cd "${MarketplaceDir}/"
    pkill -f "dfx start"
    sleep 2
    rm -fr .dfx
    rm -fr deploy 2>/dev/null
    mkdir deploy
    dfx start --clean --background
    sleep 5
fi

if [[ "$BUILD" ]]
then
    ################## BUILD ##################

    ## build cap
    #cd $CAPDir
    #dfx canister create ic-history-router
    #dfx build ic-history-router
#
    ## build dip711
    #cd $DIP721Dir
    #dfx canister create nft
    #dfx build nft
#
    ## build dip20
    #cd $DIP20Dir
    #dfx canister create wicp
    #dfx build wicp

    # build dab
    #cd $DabDir
    #dfx canister create nft
    #dfx build nft

    # build ext
    #cd $ExtDir
    #dfx canister create erc721
    #dfx build erc721
#
    # build marketplace
    cd $MarketplaceDir
    dfx canister create marketplace
    dfx build marketplace
fi

if [[ "$DEPLOY" ]]
then
    ################## DEPLOY ##################

    cd $MarketplaceDir

    # deploy cap
    #cp ${CAPDir}/target/wasm32-unknown-unknown/release/ic_history_router.wasm "${MarketplaceDir}/deploy/capmp.wasm"
    #cp ${CAPDir}/candid/router.did ${MarketplaceDir}/deploy/capmp.did
    #dfx canister create capmp
    #dfx canister install --mode=reinstall capmp
    CAPId=`dfx canister id capmp`

    # deploy dip721mp
    #cp ${DIP721Dir}/target/wasm32-unknown-unknown/release/nft.wasm "${MarketplaceDir}/deploy/dip721mp.wasm"
    #cp ${DIP721Dir}/nft/candid/nft.did ${MarketplaceDir}/deploy/dip721mp.did
    #dfx canister create dip721mp
    #dfx canister install --mode=reinstall dip721mp --argument="(principal \"${DefaultPrincipalId}\", \"CRW\", \"Crown\", principal \"${CAPId}\")"
    DIP721Id=`dfx canister id dip721mp`

    # deploy dip20mp
    #cp ${DIP20Dir}/target/wasm32-unknown-unknown/release/opt.wasm "${MarketplaceDir}/deploy/dip20mp.wasm"
    #cp ${DIP20Dir}/src/wicp.did "${MarketplaceDir}/deploy/dip20mp.did"
    #dfx canister create dip20mp
    #dfx canister install --mode=reinstall dip20mp --argument="(\"No logo\",\"Test coin\", \"TCI\", 9, 1000000, principal \"${AlicePrincipalId}\", 1, principal \"${AlicePrincipalId}\", principal \"${CAPId}\")"
    DIP20Id=`dfx canister id dip20mp`

    #deploy dabnftmp
    #cp ${DabDir}/target/wasm32-unknown-unknown/release/nft-opt.wasm "${MarketplaceDir}/deploy/dabnftmp.wasm"
    #cp ${DabDir}/candid/nft.did "${MarketplaceDir}/deploy/dabnftmp.did"
    #dfx canister create dabnftmp
    #dfx canister install --mode=reinstall dabnftmp
    DabNftId=`dfx canister id dabnftmp`

    # deploy ext721mp
    #cp ${ExtDir}/.dfx/local/canisters/erc721/erc721.wasm "${MarketplaceDir}/deploy/ext721mp.wasm"
    #cp ${ExtDir}/.dfx/local/canisters/erc721/erc721.did "${MarketplaceDir}/deploy/ext721mp.did"
    #dfx canister create ext721mp
    #dfx canister install --mode=reinstall ext721mp --argument="(principal \"${DefaultPrincipalId}\")"
    Ext721Id=`dfx canister id ext721mp`

    # deploy marketplace
    dfx canister create marketplace
    dfx canister install --mode=reinstall marketplace --argument="(principal \"${CAPId}\", principal \"$DefaultPrincipalId\")"
    MarketplaceId=`dfx canister id marketplace`

else

    CAPId=`dfx canister id capmp`
    DIP20Id=`dfx canister id dip20mp`
    DIP721Id=`dfx canister id dip721mp`
    DabNFTId=`dfx canister id dabnftmp`
    Ext721Id=`dfx canister id dabnftmp`
    MarketplaceId=`dfx canister id marketplace`
fi

if [[ "$QUICKINIT" ]]
then
    ################## INITIALIZE CANISTERS ##################

    echo "############### Alice transfers money to Bob"
    DIP20Exec "transfer Alice Bob 10000"



    echo "############### Mint 1 NFTs for Alice"
    DIP721Exec "mintDip721 Alice"

    echo "############### Allow transfers of all NFTs to MP"

    echo "############### Alice allows DIP20 transfer to the MP"
    DIP20Exec "approveExactTo Alice ${MarketplaceId} 1000000000"
fi

if [[ "$QUICKTEST" ]]
then
################## TEST CANISTERS ##################
    echo "No tests yet"
fi

if [[ "$INIT" ]]
then
    ################## INITIALIZE CANISTERS ##################

    echo "############### Alice transfers money to Bob and Charlie"
    DIP20Exec "transfer Alice Bob 10000"
    DIP20Exec "transfer Alice Charlie 10000"

    echo "############### Mint 1 NFTs for everyone"
    DIP721Exec "mintDip721 Alice"
    DIP721Exec "mintDip721 Bob"
    DIP721Exec "mintDip721 Charlie"

    echo "############### Allow transfers of all NFTs to MP"

    echo "############### Everyone allows DIP20 transfer to the MP"
    DIP20Exec "approveExactTo Alice ${MarketplaceId} 1000000000"
    DIP20Exec "approveExactTo Bob ${MarketplaceId} 1000000000"
    DIP20Exec "approveExactTo Charlie ${MarketplaceId} 1000000000"
fi

if [[ "$TEST" ]]
then
################## TEST CANISTERS ##################
    echo "No tests yet"
fi

#play -q $FinishedSound
#play -q $FinishedSound
