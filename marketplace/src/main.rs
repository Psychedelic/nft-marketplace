#![allow(warnings)]

use crate::fungible_proxy::*;
use crate::non_fungible_proxy::*;
use crate::types::*;
use crate::utils::*;
use crate::vendor_types::*;
use compile_time_run::{run_command, run_command_str};

use cap_sdk::{
    handshake, insert_sync, DetailValue, Event, IndefiniteEvent, IndefiniteEventBuilder, TypedEvent,
};
use ic_kit::{
    candid::{candid_method, encode_args, CandidType, Deserialize, Nat},
    ic,
    interfaces::{
        management::{self, CanisterStatus, CanisterStatusResponse, WithCanisterId},
        Method,
    },
    macros::*,
    Principal, RejectionCode,
};

use std::cmp::{max, min};
use std::collections::HashMap;
use std::default::Default;

mod fungible_proxy;
mod non_fungible_proxy;
mod types;
mod utils;
mod vendor_types;

#[init]
#[candid_method(init)]
pub fn init(owner: Principal, protocol_fee: Nat, cap: Option<Principal>) {
    ic_kit::ic::store(InitData {
        cap,
        owner,
        protocol_fee,
    });
    handshake(1_000_000_000_000, cap);
}

// cover metadata
#[query(name = "gitCommitHash")]
#[candid_method(query, rename = "gitCommitHash")]
fn git_commit_hash() -> &'static str {
    run_command_str!("git", "rev-parse", "HEAD")
}

#[query(name = "rustToolchainInfo")]
#[candid_method(query, rename = "rustToolchainInfo")]
fn rust_toolchain_info() -> &'static str {
    run_command_str!("rustup", "show")
}

#[query(name = "dfxInfo")]
#[candid_method(query, rename = "dfxInfo")]
fn dfx_info() -> &'static str {
    run_command_str!("dfx", "--version")
}

#[query]
#[candid_method(query)]
fn failed_log() -> Vec<TxLogEntry> {
    balances(|balances| balances.failed_tx_log_entries.clone())
}

#[update]
#[candid_method(update)]
async fn fix_balance(fungible_canister_id: Principal, user: Principal, amount: Nat) -> MPApiResult {
    if let Err(e) = is_controller(&ic::caller()).await {
        return Err(MPApiError::Unauthorized);
    }

    balances_mut(|balances| {
        *balances
            .balances
            .entry((fungible_canister_id, user))
            .or_default() = amount.clone();
    });

    Ok(())
}

/// Check if a given principal is included in the current canister controller list
///
/// To let the canister call the `aaaaa-aa` Management API `canister_status`,
/// the canister needs to be a controller of itself.
pub async fn is_controller(principal: &Principal) -> Result<(), String> {
    let caller = ic::caller();
    let self_id = ic::id();

    let status = CanisterStatus::perform(
        Principal::management_canister(),
        (WithCanisterId {
            canister_id: ic::id(),
        },),
    )
    .await
    .map(|(status,)| Ok(status))
    .unwrap_or_else(|(code, message)| Err(format!("Code: {:?}, Message: {}", code, message)))?;

    match status.settings.controllers.contains(&caller) {
        true => Ok(()),
        false => Err(format!("{} is not a controller of {}", caller, self_id)),
    }
}

/// process fees and add amounts to the fee to's balances
///
/// * `fungible_canister_id` - Principal for the fungible contract used to disperse fees in
/// * `price` - Nat amount
/// * `fees` - Vec of fees, (string fee purpose, principal of fee recipient, percent (e2))
pub fn process_fees(
    fungible_canister_id: Principal,
    price: Nat,
    fees: Vec<(String, Principal, Nat)>,
) -> Nat {
    let mut total_fee: Nat = Nat::from(0);

    for (_, principal, fee) in fees {
        // divide by 100 * 100 to allow 2 digits of precision in the fee percentage
        let amount: Nat = (price.clone() * fee.clone()) / Nat::from(10000);

        total_fee += amount.clone();

        // credit the fee to the fee recipient
        balances_mut(|balances| {
            *balances
                .balances
                .entry((fungible_canister_id, principal))
                .or_default() += amount.clone();
        });
    }

    total_fee
}

// QUERY METHODS //

/// Get the base fee for all transactions. This is stored and processed as an e2
/// For example, for a 2.5% fee (as per jelly) this value would equal 250:nat
#[query(name = "getProtocolFee")]
#[candid_method(query, rename = "getProtocolFee")]
pub async fn get_protocol_fee() -> Nat {
    init_data(|init_data| init_data.clone())
        .protocol_fee
        .clone()
}

/// Get the current registered collection data
/// Returns fungible canister, standard types, fees attached to the collection, total volume, etc
#[query(name = "getCollections")]
#[candid_method(query, rename = "getCollections")]
pub async fn get_collections() -> HashMap<Principal, Collection> {
    collections(|collections| collections.clone())
}

/// Get a tokens listing. Will return with `MPApiError::InvalidListing` if the listing does not exist.
#[query(name = "getTokenListing")]
#[candid_method(query, rename = "getTokenListing")]
pub async fn get_token_listing(
    nft_canister_id: Principal,
    token_id: Nat,
) -> Result<Listing, MPApiError> {
    // verify collection is registered
    let collections = collections(|collections| collections.clone());
    let collection = collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    marketplace_mut(|marketplace| {
        let listings = marketplace
            .listings
            .entry(nft_canister_id)
            .or_default()
            .clone();
        Ok(listings
            .get(&token_id)
            .ok_or(MPApiError::InvalidListing)?
            .clone())
    })

    // todo: switch to a method where we return empty or last known listing info with sold status
}

/// Get a tokens current offers. Can pass as many token ids as you want
#[query(name = "getTokenOffers")]
#[candid_method(query, rename = "getTokenOffers")]
pub async fn get_token_offers(
    nft_canister_id: Principal,
    token_ids: Vec<Nat>,
) -> HashMap<Nat, Vec<Offer>> {
    marketplace(|marketplace| {
        token_ids
            .into_iter()
            .map(|token_id| {
                (
                    token_id.clone(),
                    marketplace
                        .offers
                        .clone()
                        .entry(nft_canister_id)
                        .or_default()
                        .entry(token_id.clone())
                        .or_default()
                        .values()
                        .cloned()
                        .collect(),
                )
            })
            .collect()
    })
}

/// Get all the offers a buyer has made for a given collection
#[query(name = "getBuyerOffers")]
#[candid_method(query, rename = "getBuyerOffers")]
pub async fn get_buyer_offers(nft_canister_id: Principal, buyer: Principal) -> Vec<Offer> {
    // todo: new response type
    // let collections = collections(|collections| collections.clone());
    // let collection = collections
    //     .get(&nft_canister_id)
    //     .ok_or(MPApiError::NonExistentCollection)?;

    marketplace(|marketplace| {
        let mut offers = marketplace
            .offers
            .clone()
            .entry(nft_canister_id)
            .or_default()
            .clone();
        let listings = marketplace
            .listings
            .clone()
            .entry(nft_canister_id)
            .or_default()
            .clone();
        let token_list = marketplace
            .user_offers
            .clone()
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .or_default()
            .clone();

        let mut user_offers: Vec<Offer> = Vec::new();

        for token in token_list {
            let token_offers = offers.entry(token).or_default();
            let offer = token_offers.get(&buyer);
            match offer {
                Some(o) => user_offers.push(o.clone()),
                None => {}
            }
        }

        user_offers
    })
}

/// Get all users fungible balances held by marketplace
#[query(name = "getAllBalances")]
#[candid_method(query, rename = "getAllBalances")]
pub async fn get_all_balances() -> HashMap<(Principal, Principal), Nat> {
    balances(|b| b.balances.clone())
}

/// Get a balance of a user for fungibles held by marketplace
///
/// Frontends should check this, and display a popup that calls withdrawFungible,
/// to allow users to withdraw back into their wallet.
#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
pub async fn balance_of(pid: Principal) -> HashMap<Principal, Nat> {
    let balances = balances(|b| b.balances.clone());

    balances
        .into_iter()
        .filter_map(|((collection, principal), value)| {
            if principal == pid {
                return Some((collection, value));
            }
            return None;
        })
        .collect()
}

/// Get a collections floor price
#[query(name = getFloor)]
#[candid_method(query, rename = "getFloor")]
pub async fn get_floor(nft_canister_id: Principal) -> NatResult {
    let collections = collections(|collections| collections.clone());
    let collection = collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    marketplace(|marketplace| {
        let listings = marketplace
            .listings
            .get(&nft_canister_id)
            .ok_or(MPApiError::Other("No Listings".to_string()))?;

        if let Some((_, listing)) = listings
            .iter()
            .min_by_key(|(_, listing)| listing.price.clone())
        {
            return Ok(listing.price.clone());
        }

        let a = collection.clone();

        return Err(MPApiError::Other("No Listings".to_string()));
    })
}

// UPDATE METHODS //

/// Add a Collection
/// * `owner` - principal id of the owner of the collection, collection fees will be distributed to this user
/// * collection_fee` - fee multiplied by e2, for precision. `2.50% fee` = `250:nat`
/// * collection_name` - string representing the collection name. Should match dab registry entry
/// * nft_canister_id` - principal of the nft collection
/// * nft_canister_standard` - nft standard, eg; `DIP721v2`
/// * fungible_canister_id` - principal of the fungible a collection is traded with
/// * fungible_canister_standard` - fungible standard, eg; `DIP20`
#[update(name = "addCollection")]
#[candid_method(update, rename = "addCollection")]
pub async fn add_collection(
    owner: Principal,
    collection_fee: Nat,
    creation_time: u64,
    collection_name: String,
    nft_canister_id: Principal,
    nft_canister_standard: NFTStandard,
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) -> MPApiResult {
    if let Err(e) = is_controller(&ic::caller()).await {
        return Err(MPApiError::Unauthorized);
    }

    collections_mut(|collections| {
        collections.insert(
            nft_canister_id,
            Collection::new(
                owner,
                collection_fee,
                creation_time,
                collection_name,
                nft_canister_id,
                nft_canister_standard,
                fungible_canister_id,
                fungible_canister_standard,
                Nat::from(0),
            ),
        );
    });

    Ok(())
}

/// Set the base protocol level transaction fee
/// fee is stored as an e2, so for a 2.5% fee the value would be `250:nat`
#[update(name = "setProtocolFee")]
#[candid_method(update, rename = "setProtocolFee")]
async fn set_protocol_fee(fee: Nat) -> MPApiResult {
    if let Err(e) = is_controller(&ic::caller()).await {
        return Err(MPApiError::Unauthorized);
    }
    // commit to state
    init_data_mut(|init_data| {
        init_data.protocol_fee = fee;
    });

    Ok(())
}

/// Make a listing for a nft
/// price is a Nat, that should be handled as an e^n, n being the fungible canister's decimals.
/// For example, to make a 3.14 WICP offer, the number would be 3.14e8 = 314_000_000
#[update(name = "makeListing")]
#[candid_method(update, rename = "makeListing")]
pub async fn make_listing(nft_canister_id: Principal, token_id: Nat, price: Nat) -> MPApiResult {
    let collections = collections(|collections| collections.clone());
    let collection = collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let seller = ic::caller();
    let self_id = ic::id();
    let init_data = init_data(|init_data| init_data.clone());

    // check if the NFT is owned by the seller still
    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    // check if caller/seller is the token owner
    match token_owner {
        Some(principal) => {
            if (principal != seller) {
                return Err(MPApiError::Unauthorized);
            }
        }
        None => return Err(MPApiError::Unauthorized),
    }

    // check if mp is the operator still
    let token_operator = operator_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    // check if caller/seller is the token owner
    match token_operator {
        Some(principal) => {
            if (principal != self_id) {
                return Err(MPApiError::InvalidOperator);
            }
        }
        None => return Err(MPApiError::InvalidOperator),
    }

    // commit to state
    marketplace_mut(|mp| {
        let mut listing = mp
            .listings
            .entry(nft_canister_id)
            .or_default()
            .entry(token_id.clone())
            .or_default();

        if (listing.status == ListingStatus::Selling) {
            return Err(MPApiError::InvalidListingStatus);
        }

        *listing = Listing::new(
            price.clone(),
            seller,
            ListingStatus::Created,
            ic::time(),
            [
                (
                    "Protocol Fee".to_string(),
                    init_data.owner,
                    init_data.protocol_fee.clone(),
                ),
                (
                    "Collection Fee".to_string(),
                    collection.owner,
                    collection.collection_fee.clone(),
                ),
            ]
            .to_vec(),
        );

        // insert (async with fallback) event to cap
        insert_sync(
            IndefiniteEventBuilder::new()
                .caller(seller)
                .operation("makeListing")
                .details(vec![
                    (
                        "token_id".into(),
                        DetailValue::U64(convert_nat_to_u64(token_id).unwrap()),
                    ),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(collection.nft_canister_id),
                    ),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(price.clone()).unwrap()),
                    ),
                    ("seller".into(), DetailValue::Principal(seller)),
                ])
                .build()
                .unwrap(),
        );

        Ok(())
    })
}

/// Make an offer on a given nft
///
/// * `price` - Nat that should be handled as an e^n, n being the fungible canister's decimals.
/// For example, to make a 3.14 WICP offer, the number would be 3.14e8 = 314_000_000
///
/// The caller should have an allowance set for marketplace  for the given fungible canister, that is
/// equal to the total of all offers made already, plus the price for the current offer. For example,
/// if a user has made 2 offers for 1.00 WICP each, and is making an additional offer of 1.00 WICP,
/// the total allowance should be 3 WICP.
#[update(name = "makeOffer")]
#[candid_method(update, rename = "makeOffer")]
pub async fn make_offer(nft_canister_id: Principal, token_id: Nat, price: Nat) -> MPApiResult {
    let collections = collections(|collections| collections.clone());
    let collection = collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let buyer = ic::caller();
    let self_id = ic::id();

    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?
    .ok_or(MPApiError::Other("error calling owner_of".to_string()))?;

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &buyer,
        &self_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .map_err(|_| MPApiError::Other("Error calling allowance".to_string()))?;

    if allowance.clone() < price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .map_err(|_| MPApiError::Other("Error calling balanceOf".to_string()))?;

    if balance.clone() < price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    // commit to state
    marketplace_mut(|mp| {
        let offers = mp
            .offers
            .entry(nft_canister_id)
            .or_default()
            .entry(token_id.clone())
            .or_default();

        offers
            .entry(buyer)
            .and_modify(|offer| {
                // listing already exists, we are modifying it here
                offer.price = price.clone();
            })
            .or_insert_with(|| {
                Offer::new(
                    nft_canister_id,
                    token_id.clone(),
                    price.clone(),
                    buyer,
                    token_owner,
                    OfferStatus::Created,
                    ic::time(),
                )
            });

        let buyer_offers = mp
            .user_offers
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .or_default();
        if !buyer_offers.contains(&token_id.clone()) {
            buyer_offers.push(token_id.clone());
        }
    });

    // insert (async with fallback) event to cap
    insert_sync(
        IndefiniteEventBuilder::new()
            .caller(buyer)
            .operation("makeOffer")
            .details(vec![
                (
                    "token_id".into(),
                    DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                ),
                (
                    "nft_canister_id".into(),
                    DetailValue::Principal(nft_canister_id),
                ),
                (
                    "price".into(),
                    DetailValue::U64(convert_nat_to_u64(price.clone()).unwrap()),
                ),
                ("buyer".into(), DetailValue::Principal(buyer)),
                ("seller".into(), DetailValue::Principal(token_owner)),
            ])
            .build()
            .unwrap(),
    );

    Ok(())
}

/// Direct buy a nft that has been listed
///
/// * `nft_canister_id` - principal id of the nft collection contract
/// * `token_id` - Token to purchase
///
/// ## Integrating
///
/// To use this method, an allowance for marketplace must be set prior to calling this.
/// This fungtion will check to make sure the neccessary requirements are fullfilled, then auto-withdraw
/// the fungible amount from the buyer to the marketplace canister. After the funds are secured,
/// a `transferFrom` call is made to send the nft to the buyer. On success, fungible amounts are
/// released to the respective principal ids for the fee recipients and the seller. In the slim case where
/// a transaction passed all checks, but an error occurred transferring the nft, the buyers balance will
/// remain on the marketplace for withdraw using the `withdrawFungible` as a fallback
#[update(name = "directBuy")]
#[candid_method(update, rename = "directBuy")]
pub async fn direct_buy(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let c = collections(|collections| collections.clone());
    let collection = c
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let buyer = ic::caller();
    let self_id = ic::id();

    // check listing exists
    let mut all_listings = marketplace(|mp| mp.listings.clone());

    let listings = all_listings.entry(nft_canister_id).or_default();

    let listing = listings
        .get_mut(&token_id.clone())
        .ok_or(MPApiError::InvalidListing)?;

    // guarding against re-entrancy
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    let price = listing.price.clone();

    // check token owner and operator
    let token_owner: Principal;
    let token_metadata = DIP721v2Proxy::token_metadata(&token_id.clone(), &nft_canister_id).await;
    match token_metadata {
        Ok(metadata) => {
            match metadata.owner {
                Some(principal) => {
                    // we only care if mp is the operator, disregard current owner/listing
                    token_owner = principal;
                }
                None => return Err(MPApiError::InvalidOwner),
            }
            match metadata.operator {
                Some(principal) => {
                    if (principal != self_id) {
                        return Err(MPApiError::InvalidOperator);
                    }
                }
                None => return Err(MPApiError::InvalidOperator),
            }
        }
        Err(e) => return Err(e),
    }

    // Claim funds from user wallet
    match transfer_from_fungible(
        &buyer,
        &self_id,
        &price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    {
        Err(e) => return Err(e),
        Ok(_) => {}
    }

    // Successfully auto deposited fungibles, transfer the nft from marketplace to the buyer
    match transfer_from_non_fungible(
        &token_owner,                     // from
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    {
        Err(e) => {
            // error transferring nft, sale failed

            // send funds back to buyer
            match transfer_fungible(
                &buyer,
                &price.clone(),
                &collection.fungible_canister_id,
                collection.fungible_canister_standard.clone(),
            )
            .await
            {
                Err(e) => {
                    // auto withdraw failed, fallback to withdrawFungible
                    // add deposited funds to buyer mp balance (fallback to avoid extra transactions/time/cycles)
                    balances_mut(|balances| {
                        *balances
                            .balances
                            .entry((collection.fungible_canister_id, buyer))
                            .or_default() += price.clone();
                    });
                }
                Ok(_) => {}
            }

            balances_mut(|balances| {
                balances.failed_tx_log_entries.push(TxLogEntry::new(
                buyer.clone(),
                token_owner.clone(),
                format!(
"direct buy non fungible failed for user {} for contract {} for token id {}; price {:?}; error: {:?}",
buyer, nft_canister_id, token_id, price.clone(), e,
)));
            });

            return Err(e);
        }
        Ok(_) => {}
    }

    let total_fees = process_fees(
        collection.fungible_canister_id,
        price.clone(),
        listing.fee.clone(),
    );

    // transfer the funds from the MP to the seller, or
    if transfer_fungible(
        &token_owner,
        &(price.clone() - total_fees.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // fallback to sellers mp balance
        balances_mut(|balances| {
            *balances
                .balances
                .entry((collection.fungible_canister_id, token_owner))
                .or_default() += price.clone() - total_fees.clone();
        });
    }

    // commit to state
    marketplace_mut(|mp| {
        // remove listing
        listings.remove(&token_id.clone());

        // remove offer from user if exists
        mp.offers
            .entry(nft_canister_id)
            .or_default()
            .entry(token_id.clone())
            .or_default()
            .remove(&buyer);

        mp.user_offers
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .and_modify(|tokens| {
                tokens.retain(|token| token != &token_id.clone());
            })
            .or_default();
    });

    // update market cap for collection
    collections_mut(|collections| {
        collections
            .entry(nft_canister_id)
            .and_modify(|collection_data| {
                collection_data.fungible_volume =
                    collection_data.fungible_volume.clone() + price.clone();
            });
    });

    // insert (async with fallback) event to cap
    insert_sync(
        IndefiniteEventBuilder::new()
            .caller(buyer)
            .operation("directBuy")
            .details(vec![
                (
                    "token_id".into(),
                    DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                ),
                (
                    "nft_canister_id".into(),
                    DetailValue::Principal(nft_canister_id),
                ),
                ("buyer".into(), DetailValue::Principal(buyer)),
                ("seller".into(), DetailValue::Principal(token_owner)),
                (
                    "price".into(),
                    DetailValue::U64(convert_nat_to_u64(price.clone()).unwrap()),
                ),
                (
                    "total_fees".into(),
                    DetailValue::U64(convert_nat_to_u64(total_fees.clone()).unwrap()),
                ),
            ])
            .build()
            .unwrap(),
    );

    Ok(())
}

/// Accept an offer that has been made on any given nft
#[update(name = "acceptOffer")]
#[candid_method(update, rename = "acceptOffer")]
pub async fn accept_offer(
    nft_canister_id: Principal,
    token_id: Nat,
    buyer: Principal,
) -> MPApiResult {
    let c = collections(|collections| collections.clone());
    let collection = c
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let seller = ic::caller();
    let self_id = ic::id();

    let mut offers = marketplace(|mp| mp.offers.clone());
    let token_offers = offers
        .entry(nft_canister_id)
        .or_default()
        .entry(token_id.clone())
        .or_default();
    let offer = token_offers.get(&buyer).ok_or(MPApiError::InvalidListing)?;
    let offer_price = offer.price.clone();

    let init_data = init_data(|init_data| init_data.clone());

    // guarding against re-entrancy
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    // check token owner and operator
    let token_owner: Principal;
    let token_metadata = DIP721v2Proxy::token_metadata(&token_id.clone(), &nft_canister_id).await;
    match token_metadata {
        Ok(metadata) => {
            match metadata.owner {
                Some(principal) => {
                    // error if caller is not the token owner
                    if (principal != seller) {
                        return Err(MPApiError::Unauthorized);
                    }
                    token_owner = principal;
                }
                None => return Err(MPApiError::InvalidOwner),
            }
            match metadata.operator {
                Some(principal) => {
                    if (principal != self_id) {
                        return Err(MPApiError::InvalidOperator);
                    }
                }
                None => return Err(MPApiError::InvalidOperator),
            }
        }
        Err(e) => return Err(e),
    }

    // Claim funds from user wallet
    match transfer_from_fungible(
        &buyer,
        &self_id,
        &offer_price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    {
        Err(e) => return Err(e),
        Ok(_) => {}
    }

    // Successfully auto deposited fungibles, transfer the nft from marketplace to the buyer
    match transfer_from_non_fungible(
        &seller,                          // from
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    {
        Err(e) => {
            // error transferring nft, sale failed

            // send funds back to buyer
            match transfer_fungible(
                &buyer,
                &offer_price.clone(),
                &collection.fungible_canister_id,
                collection.fungible_canister_standard.clone(),
            )
            .await
            {
                Err(e) => {
                    // auto withdraw failed, fallback to withdrawFungible
                    // add deposited funds to buyer mp balance (fallback to avoid extra transactions/time/cycles)
                    balances_mut(|balances| {
                        *balances
                            .balances
                            .entry((collection.fungible_canister_id, buyer))
                            .or_default() += offer_price.clone();
                    });
                }
                Ok(_) => {}
            }

            balances_mut(|balances| {
                balances.failed_tx_log_entries.push(TxLogEntry::new(
                seller.clone(),
                buyer.clone(),
                format!(
"accept offer non fungible failed for user {} for contract {} for token id {}; price: {:?}; error: {:?}",
seller, nft_canister_id, token_id, offer_price.clone(), e,
)));
            });
            return Err(e);
        }
        Ok(_) => {}
    }

    let total_fees = process_fees(
        collection.fungible_canister_id,
        offer_price.clone(),
        [
            (
                "Protocol Fee".to_string(),
                init_data.owner,
                init_data.protocol_fee.clone(),
            ),
            (
                "Collection Fee".to_string(),
                collection.owner,
                collection.collection_fee.clone(),
            ),
        ]
        .to_vec(),
    );

    // successfully transferred nft to buyer, release funds to seller
    if transfer_fungible(
        &seller,
        &(offer_price.clone() - total_fees.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // add deposited funds to seller balance
        balances_mut(|balances| {
            *balances
                .balances
                .entry((collection.fungible_canister_id, seller))
                .or_default() += offer_price.clone() - total_fees.clone();
        });
    }

    // commit to state
    marketplace_mut(|mp| {
        let offers = mp
            .offers
            .entry(nft_canister_id)
            .or_default()
            .entry(token_id.clone())
            .or_default();

        let listings = mp.listings.entry(nft_canister_id).or_default();

        let listing = listings.get_mut(&token_id.clone());

        // remove listing and offer
        listings.remove(&token_id.clone());
        offers.remove(&buyer);

        // avoid keeping an empty object for a token if no more offers
        if (offers.len() == 0) {
            mp.offers
                .entry(nft_canister_id)
                .or_default()
                .remove(&token_id.clone());
        }

        mp.user_offers
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .and_modify(|tokens| {
                tokens.retain(|token| token != &token_id.clone());
            })
            .or_default();
    });

    // update market cap for collection
    collections_mut(|collections| {
        collections
            .entry(nft_canister_id)
            .and_modify(|collection_data| {
                collection_data.fungible_volume =
                    collection_data.fungible_volume.clone() + offer_price.clone();
            });
    });

    // insert (async with fallback) event to cap
    insert_sync(
        IndefiniteEventBuilder::new()
            .caller(seller)
            .operation("acceptOffer")
            .details(vec![
                (
                    "token_id".into(),
                    DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                ),
                (
                    "nft_canister_id".into(),
                    DetailValue::Principal(nft_canister_id),
                ),
                ("buyer".into(), DetailValue::Principal(buyer)),
                ("seller".into(), DetailValue::Principal(seller)),
                (
                    "price".into(),
                    DetailValue::U64(convert_nat_to_u64(offer_price.clone()).unwrap()),
                ),
                (
                    "total_fees".into(),
                    DetailValue::U64(convert_nat_to_u64(total_fees).unwrap()),
                ),
            ])
            .build()
            .unwrap(),
    );

    Ok(())
}

/// Cancel a created listing
#[update(name = "cancelListing")]
#[candid_method(update, rename = "cancelListing")]
pub async fn cancel_listing(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let c = collections(|collections| collections.clone());
    let collection = c
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // commit to state
    marketplace_mut(|mp| {
        let seller = ic::caller();

        let listings = mp.listings.entry(nft_canister_id).or_default();

        let listing = listings
            .get(&token_id.clone())
            .ok_or(MPApiError::InvalidListing)?
            .clone();

        if (seller != listing.seller) {
            return Err(MPApiError::Unauthorized);
        }

        if listing.status != ListingStatus::Created {
            return Err(MPApiError::InvalidListingStatus);
        }

        listings.remove(&token_id.clone());

        // insert (async with fallback) event to cap
        insert_sync(
            IndefiniteEventBuilder::new()
                .caller(seller)
                .operation("cancelListing")
                .details(vec![
                    (
                        "token_id".into(),
                        DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                    ),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(nft_canister_id),
                    ),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(listing.price.clone()).unwrap()),
                    ),
                    ("seller".into(), DetailValue::Principal(seller)),
                ])
                .build()
                .unwrap(),
        );

        Ok(())
    })
}

/// Cancel a created offer
#[update(name = "cancelOffer")]
#[candid_method(update, rename = "cancelOffer")]
pub async fn cancel_offer(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let c = collections(|collections| collections.clone());
    let collection = c
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?
    .ok_or(MPApiError::Other("error calling owner_of".to_string()))?;

    // commit to state
    marketplace_mut(|mp| {
        let buyer = ic::caller();

        let mut offers = mp
            .offers
            .entry(nft_canister_id)
            .or_default()
            .entry(token_id.clone())
            .or_default();

        let offer = offers
            .get(&buyer)
            .ok_or(MPApiError::InvalidListing)?
            .clone();

        offers.remove(&buyer);

        if (offers.len() == 0) {
            mp.offers
                .entry(nft_canister_id)
                .or_default()
                .remove(&token_id.clone());
        }

        mp.user_offers
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .and_modify(|tokens| {
                tokens.retain(|token| token != &token_id.clone());
            })
            .or_default();

        // insert (async with fallback) event to cap
        insert_sync(
            IndefiniteEventBuilder::new()
                .caller(buyer)
                .operation("cancelOffer")
                .details(vec![
                    (
                        "token_id".into(),
                        DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                    ),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(nft_canister_id),
                    ),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(offer.price.clone()).unwrap()),
                    ),
                    ("buyer".into(), DetailValue::Principal(buyer)),
                    ("seller".into(), DetailValue::Principal(token_owner)),
                ])
                .build()
                .unwrap(),
        );

        Ok(())
    })
}

/// Deny an offer made to an owned nft
///
/// - todo: this is a seller/nft ownerd method, update variable names and verify that
#[update(name = "denyOffer")]
#[candid_method(update, rename = "denyOffer")]
pub async fn deny_offer(
    nft_canister_id: Principal,
    token_id: Nat,
    buyer: Principal,
) -> MPApiResult {
    let collections = collections(|collections| collections.clone());
    let collection = collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let seller = ic::caller();

    // check the caller is the owner of the nft
    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?
    .ok_or(MPApiError::Other("error calling owner_of".to_string()))?;

    if (seller != token_owner) {
        return Err(MPApiError::Unauthorized);
    }

    let mut offers = marketplace(|mp| mp.offers.clone());
    let mut token_offers = offers
        .entry(nft_canister_id)
        .or_default()
        .entry(token_id.clone())
        .or_default();

    let offer = token_offers
        .get(&buyer)
        .ok_or(MPApiError::InvalidOffer)?
        .clone();

    // commit to state
    marketplace_mut(|mp| {
        mp.offers.remove(&buyer);

        if (offers.len() == 0) {
            mp.offers
                .entry(nft_canister_id)
                .or_default()
                .remove(&token_id.clone());
        }

        mp.user_offers
            .entry(buyer)
            .or_default()
            .entry(nft_canister_id)
            .and_modify(|tokens| {
                tokens.retain(|token| token != &token_id.clone());
            })
            .or_default();
    });

    // insert (async with fallback) event to cap
    insert_sync(
        IndefiniteEventBuilder::new()
            .caller(buyer)
            .operation("denyOffer")
            .details(vec![
                (
                    "token_id".into(),
                    DetailValue::U64(convert_nat_to_u64(token_id.clone()).unwrap()),
                ),
                (
                    "nft_canister_id".into(),
                    DetailValue::Principal(nft_canister_id),
                ),
                (
                    "price".into(),
                    DetailValue::U64(convert_nat_to_u64(offer.price.clone()).unwrap()),
                ),
                ("buyer".into(), DetailValue::Principal(buyer)),
            ])
            .build()
            .unwrap(),
    );

    Ok(())
}

/// Withdraw Fungible
///
/// this is a fallback method, for withdrawing held fungibles in the marketplace canister.
#[update(name = "withdrawFungible")]
#[candid_method(update, rename = "withdrawFungible")]
pub async fn withdraw_fungible(
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let mut empty = false;

    let balances = balances(|balances| balances.balances.clone());
    let balance = balances
        .get(&(fungible_canister_id, caller))
        .ok_or(MPApiError::InsufficientFungibleBalance)?;

    if balance.clone() <= Nat::from(0) {
        return Err(MPApiError::InsufficientFungibleBalance);
    }
    if transfer_fungible(
        &caller,
        balance,
        &fungible_canister_id,
        fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        balances_mut(|balances| {
            balances.failed_tx_log_entries.push(TxLogEntry::new(
                self_id,
                caller.clone(),
                format!("withdraw failed for user {}", caller,),
            ));
        });

        return Err(MPApiError::TransferFungibleError);
    }

    // remove balance
    balances_mut(|balances| {
        balances.balances.remove(&(fungible_canister_id, caller));
    });

    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

candid::export_service!();

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    std::print!("{}", __export_service());
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
