#![allow(warnings)]

use crate::fungible_proxy::*;
use crate::non_fungible_proxy::*;
use crate::types::*;
use crate::utils::*;
use crate::vendor_types::*;

use cap_sdk::{
    handshake, insert, DetailValue, Event, IndefiniteEvent, IndefiniteEventBuilder, TypedEvent,
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
pub fn init(cap: Principal, owner: Principal, protocol_fee: Nat) {
    ic_kit::ic::store(InitData {
        cap,
        owner,
        protocol_fee,
    });
    handshake(1_000_000_000_000, Some(cap));
}

// to let the canister call the `aaaaa-aa` Management API `canister_status`
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

pub fn process_fees(fungible_canister_id: Principal, price: Nat, fees: Vec<(String, Principal, Nat)>) -> Nat {
    let mut total_fee: Nat = Nat::from(0);

    for (_, principal, fee) in fees {
        // divide by 10000 to allow 2 digits of precision in the fee
        let amount: Nat =
            price.clone() * fee.clone() / Nat::from(10000);

        total_fee += amount.clone();

        // credit the owner fee to the collection owner
        *balances()
            .balances
            .entry((fungible_canister_id, principal))
            .or_default() += amount.clone();
    }
    
    total_fee
}

// QUERY METHODS //

#[query(name = "getCollections")]
#[candid_method(query, rename = "getCollections")]
pub async fn get_collections(
    nft_canister_id: Principal,
    token_id: Nat,
) -> Vec<Collection> {
    collections().collections.clone().values().cloned().collect()
}

#[query(name = "getTokenListing")]
#[candid_method(query, rename = "getTokenListing")]
pub async fn get_token_listing(
    nft_canister_id: Principal,
    token_id: Nat,
) -> Result<Listing, MPApiError> {
    // verify collection is registered
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let listings = marketplace()
        .listings
        .entry(nft_canister_id)
        .or_default()
        .clone();
    Ok(listings
        .get(&token_id)
        .ok_or(MPApiError::InvalidListing)?
        .clone())
}

#[query(name = "getTokenOffers")]
#[candid_method(query, rename = "getTokenOffers")]
pub async fn get_token_offers(
    nft_canister_id: Principal,
    token_ids: Vec<Nat>,
) -> HashMap<Nat, Vec<Offer>> {
    token_ids
        .into_iter()
        .map(|token_id| {
            (
                token_id.clone(),
                marketplace()
                    .offers
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
}

#[query(name = "getBuyerOffers")]
#[candid_method(query, rename = "getBuyerOffers")]
pub async fn get_buyer_offers(nft_canister_id: Principal, buyer: Principal) -> Vec<Offer> {
    let mut offers = marketplace()
        .offers
        .entry(nft_canister_id)
        .or_default()
        .clone();
    let listings = marketplace().listings.entry(nft_canister_id).or_default().clone();
    let token_list = marketplace()
        .user_offers
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
            None => {},
        }
    }

    user_offers
}

#[query(name = "getAllBalances")]
#[candid_method(query, rename = "getAllBalances")]
pub async fn get_all_balances() -> HashMap<(Principal, Principal), Nat> {
    let bals = &balances().balances;
    bals.clone()
}

#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
pub async fn balance_of(pid: Principal) -> HashMap<Principal, Nat> {
    let balances = balances().balances.clone();

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

#[query(name = getFloor)]
#[candid_method(query, rename = "getFloor")]
pub async fn get_floor(nft_canister_id: Principal) -> NatResult {
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let listings = marketplace()
        .listings
        .get(&nft_canister_id)
        .ok_or(MPApiError::Other("No Listings".to_string()))?;

    if let Some((_, listing)) = listings
        .iter()
        .min_by_key(|(_, listing)| listing.price.clone())
    {
        return Ok(listing.price.clone());
    }

    return Err(MPApiError::Other("No Listings".to_string()));
}

// UPDATE METHODS //

#[update(name = "addCollection")]
#[candid_method(update, rename = "addCollection")]
fn add_collection(
    owner: Principal,
    collection_fee: Nat,
    creation_time: u64,
    collection_name: String,
    nft_canister_id: Principal,
    nft_canister_standard: NFTStandard,
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) -> MPApiResult {
    assert_eq!(ic::caller(), init_data().owner);

    collections().collections.insert(
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
        ),
    );

    Ok(())
}

#[update(name = "makeListing")]
#[candid_method(update, rename = "makeListing")]
pub async fn make_listing(nft_canister_id: Principal, token_id: Nat, price: Nat) -> MPApiResult {
    // Todo: if caller is a curator pid, handle fees
    let seller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let init_data = init_data();

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
        },
        None => return Err(MPApiError::Unauthorized)
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
        },
        None => return Err(MPApiError::InvalidOperator)
    }

    let mut mp = marketplace();

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
        [(
            "Protocol Fee".to_string(),
            init_data.owner,
            init_data.protocol_fee.clone(),
        ),
        (
            "Collection Fee".to_string(),
            collection.owner,
            collection.collection_fee.clone(),
        )]
        .to_vec(),
    );

    capq()
        .insert_into_cap(
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
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "makeOffer")]
#[candid_method(update, rename = "makeOffer")]
pub async fn make_offer(nft_canister_id: Principal, token_id: Nat, price: Nat) -> MPApiResult {
    let buyer = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?.ok_or(MPApiError::Other("error calling owner_of".to_string()))?;

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &buyer,
        &self_id,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling allowance".to_string()))?;

    if allowance.clone() < price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling balanceOf".to_string()))?;

    if balance.clone() < price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

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

    capq()
        .insert_into_cap(
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
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "directBuy")]
#[candid_method(update, rename = "directBuy")]
pub async fn direct_buy(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let buyer = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let listings = mp.listings.entry(nft_canister_id).or_default();

    let listing = listings
        .get_mut(&token_id.clone())
        .ok_or(MPApiError::InvalidListing)?;

    // guarding against re-entrancy
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

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
            if (principal != listing.seller) {
                return Err(MPApiError::Unauthorized);
            }
        },
        None => return Err(MPApiError::Unauthorized)
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
        },
        None => return Err(MPApiError::InvalidOperator)
    }

    // guarding agains reentrancy
    listing.status = ListingStatus::Selling;

    // Auto deposit tokens

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &buyer,
        &self_id,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling allowance".to_string()))?;

    if allowance.clone() < listing.price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling balanceOf".to_string()))?;

    if balance.clone() < listing.price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    // auto deposit funds to mp from buyer
    if transfer_from_fungible(
        &buyer,
        &self_id,
        &listing.price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            listing.seller.clone(),
            format!(
                "accept_offer failed for user {} for contract {} for token id {}; transfer 2",
                listing.seller, nft_canister_id, token_id,
            ),
        ));
        return Err(MPApiError::TransferFungibleError);
    }

    // Successfully auto deposited fungibles, transfer the nft from marketplace to the buyer
    if transfer_from_non_fungible(
        &listing.seller,         // from
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        // add deposited funds to buyer mp balance
        *balances()
            .balances
            .entry((collection.fungible_canister_id, buyer))
            .or_default() += listing.price.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buyer.clone(),
            format!(
        "accept_offer non fungible failed for user {} for contract {} for token id {}; transfer 1",
        buyer, nft_canister_id, token_id,
      ),
        ));

        return Err(MPApiError::TransferNonFungibleError);
    }

    let total_fees = process_fees(collection.fungible_canister_id, listing.price.clone(), listing.fee.clone());
    ic::print(total_fees.to_string());

    // transfer the funds from the MP to the seller, or
    if transfer_fungible(
        &listing.seller,
        &(listing.price.clone() - total_fees.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // fallback to sellers mp balance
        *balances()
            .balances
            .entry((collection.fungible_canister_id, listing.seller))
            .or_default() += listing.price.clone() - total_fees.clone();
    }

    let price = listing.price.clone();

    // remove listing
    listings.remove(&token_id.clone());

    capq()
        .insert_into_cap(
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
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(price).unwrap()),
                    ),
                    (
                        "total_fees".into(),
                        DetailValue::U64(convert_nat_to_u64(total_fees).unwrap()),
                    ),
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "acceptOffer")]
#[candid_method(update, rename = "acceptOffer")]
pub async fn accept_offer(
    nft_canister_id: Principal,
    token_id: Nat,
    buyer: Principal,
) -> MPApiResult {
    let seller = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;
    let offers = mp
        .offers
        .entry(nft_canister_id)
        .or_default()
        .entry(token_id.clone())
        .or_default();

    let offer = offers.get_mut(&buyer).ok_or(MPApiError::InvalidListing)?;
    let offer_price = offer.price.clone();

    let init_data = init_data();

    // guarding against re-entrancy
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    let listings = mp.listings.entry(nft_canister_id).or_default();

    let listing = listings.get_mut(&token_id.clone());

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

    match token_operator {
        Some(principal) => {
            if (principal != self_id) {
                return Err(MPApiError::InvalidOperator);
            }
        }
        None => return Err(MPApiError::InvalidOperator),
    }

    if let Some(listed) = listing {
        // guarding against reentrancy
        listed.status = ListingStatus::Selling;
    }

    // Auto deposit tokens

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &buyer,
        &self_id,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling allowance".to_string()))?;

    if allowance.clone() < offer_price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await.map_err(|_| MPApiError::Other("Error calling balanceOf".to_string()))?;

    if balance.clone() < offer_price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    // auto deposit funds to mp from buyer
    if transfer_from_fungible(
        &buyer,
        &self_id,
        &offer_price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            seller.clone(),
            format!(
                "accept_offer failed for user {} for contract {} for token id {}; transfer 2",
                seller, nft_canister_id, token_id,
            ),
        ));
        return Err(MPApiError::TransferFungibleError);
    }

    // Successfully auto deposited fungibles from buyer, transfer the nft from the seller to the buyer
    if transfer_from_non_fungible(
        &seller,                          // from
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        // add deposited funds to buyer mp balance
        *balances()
            .balances
            .entry((collection.fungible_canister_id, buyer))
            .or_default() += offer_price.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buyer.clone(),
            format!(
        "accept_offer non fungible failed for user {} for contract {} for token id {}; transfer 1",
        buyer, nft_canister_id, token_id,
      ),
        ));

        return Err(MPApiError::TransferNonFungibleError);
    }

    let total_fees = process_fees(collection.fungible_canister_id, offer_price.clone(), [(
        "Protocol Fee".to_string(),
        init_data.owner,
        init_data.protocol_fee.clone(),
    ),
    (
        "Collection Fee".to_string(),
        collection.owner,
        collection.collection_fee.clone(),
    )]
    .to_vec());   
    
    // successfully transferred nft to buyer, release funds to seller
    if transfer_fungible(
        &seller,
        &(offer.price.clone() - total_fees.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // add deposited funds to buyer mp balance
        *balances()
            .balances
            .entry((collection.fungible_canister_id, buyer))
            .or_default() += offer_price.clone() - total_fees.clone();
    }    

    offer.status = OfferStatus::Bought;

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

    capq()
        .insert_into_cap(
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
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "cancelListing")]
#[candid_method(update, rename = "cancelListing")]
pub async fn cancel_listing(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let seller = ic::caller();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

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

    capq()
        .insert_into_cap(
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
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "cancelOffer")]
#[candid_method(update, rename = "cancelOffer")]
pub async fn cancel_offer(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let buyer = ic::caller();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

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

    capq()
        .insert_into_cap(
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
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "denyOffer")]
#[candid_method(update, rename = "denyOffer")]
pub async fn deny_offer(
    nft_canister_id: Principal,
    token_id: Nat,
    buyer: Principal,
) -> MPApiResult {
    let buyer = ic::caller();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

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

    capq()
        .insert_into_cap(
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
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "withdrawFungible")]
#[candid_method(update, rename = "withdrawFungible")]
pub async fn withdraw_fungible(
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let mut empty = false;

    if let Some(balance) = balances().balances.get_mut(&(fungible_canister_id, caller)) {
        let balance_to_send = balance.clone();
        if balance_to_send <= Nat::from(0) {
            return Err(MPApiError::Other(
                "Error: No unlocked tokens to withdraw".to_string(),
            ));
        }
        *balance -= balance_to_send.clone();
        if transfer_fungible(
            &caller,
            &balance_to_send,
            &fungible_canister_id,
            fungible_canister_standard.clone(),
        )
        .await
        .is_err()
        {
            *balance += balance_to_send.clone();
            balances().failed_tx_log_entries.push(TxLogEntry::new(
                self_id,
                caller.clone(),
                format!("withdraw failed for user {}", caller,),
            ));

            return Err(MPApiError::TransferFungibleError);
        }

        if (balance.clone() == 0) {
            empty = true;
        }
    } else {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    if empty {
        // cleanup empty balance
        balances().balances.remove(&(fungible_canister_id, caller));
    }

    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
