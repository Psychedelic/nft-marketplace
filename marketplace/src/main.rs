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
    interfaces::{management, Method},
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
pub fn init(cap: Principal, owner: Principal) {
    ic_kit::ic::store(InitData { cap, owner });
    handshake(1_000_000_000_000, Some(cap));
}

// LIST FOR SALE
// params:
//  `collection_id` : pid of nft collection
//  `token_id`      : id of token listing for
//  `direct_buy`    : bool, if true deposit is required with listing (check if collection owns)
//  `price`         : nat,
//
// 1) check ownership:
//    (direct_buy ? (owner(token) = marketplace) : (owner(token) = caller))
// 2) cap event:
//    token_id
//    nft collection id
//    price
//    listing uuid
#[update(name = "makeListing")]
#[candid_method(update, rename = "makeListing")]
pub async fn make_listing(
    direct_buy: bool,
    nft_canister_id: Principal,
    token_id: u64,
    price: Nat,
) -> MPApiResult {
    // Todo: if caller is a curator pid, handle fees
    let caller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard.clone(),
    )
    .await?;

    // todo direct buy support
    if (direct_buy) {
        if (token_owner.unwrap() != self_id) {
            return Err(MPApiError::NoDeposit);
        }
    } else {
        if (token_owner.unwrap() != caller) {
            return Err(MPApiError::Unauthorized);
        }
    }

    let mut mp = marketplace();

    let mut listing = mp
        .listings
        .entry((nft_canister_id, token_id.clone()))
        .or_default();

    if (listing.status == ListingStatus::Selling) {
        return Err(MPApiError::InvalidListingStatus);
    }

    *listing = Listing::new(
        direct_buy.clone(),
        price.clone(),
        caller,
        ListingStatus::Created,
    );

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("makeListing")
                .details(vec![
                    ("token_id".into(), DetailValue::U64(token_id)),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(collection.nft_canister_id),
                    ),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(price.clone()).unwrap()),
                    ),
                    ("seller".into(), DetailValue::Principal(caller)),
                    (
                        "direct_buy".into(),
                        if (direct_buy) {
                            DetailValue::True
                        } else {
                            DetailValue::False
                        },
                    ),
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
pub async fn make_offer(nft_canister_id: Principal, token_id: u64, price: Nat) -> U64Result {
    let caller = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // check if the buyer still has the money to pay
    let fungible_balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &caller,
        collection.fungible_canister_standard.clone(),
    )
    .await?;

    if (fungible_balance < price) {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    mp.offers.push(Offer::new(
        nft_canister_id,
        token_id.clone(),
        price.clone(),
        caller,
        OfferStatus::Created,
    ));
    let buy_id = mp.offers.len() as u64;

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("makeOffer")
                .details(vec![
                    ("token_id".into(), DetailValue::U64(token_id.clone())),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(nft_canister_id),
                    ),
                    ("buy_id".into(), DetailValue::U64(buy_id)),
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

    Ok(buy_id)
}

#[update(name = "acceptOffer")]
#[candid_method(update, rename = "acceptOffer")]
pub async fn accept_offer(buy_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let offer = mp
        .offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidOffer)?;

    // guarding against re-entrancy
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    let listing = mp
        .listings
        .get_mut(&(offer.nft_canister_id, offer.token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;

    // guarding against re-entrancy
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    // only the seller can accept the bid
    if (listing.payment_address != caller) {
        return Err(MPApiError::Unauthorized);
    }

    let collection = collections()
        .collections
        .get(&offer.nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // check if the NFT is still hold by the seller
    let token_owner = owner_of_non_fungible(
        &offer.nft_canister_id,
        &offer.token_id,
        collection.nft_canister_standard.clone(),
    )
    .await?;

    if (listing.payment_address != token_owner.unwrap()) {
        mp.listings
            .remove(&(offer.nft_canister_id, offer.token_id.clone()));
        return Err(MPApiError::InsufficientNonFungibleBalance);
    }

    // guarding agains reentrancy
    offer.status = OfferStatus::Bought;
    listing.status = ListingStatus::Selling;

    // transfer the money from the buyer to the MP contract
    if transfer_from_fungible(
        &offer.payment_address,
        &self_id,
        &offer.price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        offer.status = OfferStatus::Created;
        listing.status = ListingStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    // transfer the nft from the seller to the buyer
    if transfer_from_non_fungible(
        &listing.payment_address,
        &offer.payment_address,
        &offer.token_id,
        &offer.nft_canister_id,
        collection.nft_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the buyer, as he never received the NFT
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, offer.payment_address))
            .or_default()) += offer.price.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            offer.payment_address.clone(),
            format!(
                "accept_offer non fungible failed for user {} for buy id {}; transfer 1",
                offer.payment_address, buy_id
            ),
        ));

        offer.status = OfferStatus::Created;
        listing.status = ListingStatus::Created;
        return Err(MPApiError::TransferNonFungibleError);
    }

    let owner_fee: Nat = offer.price.clone() * collection.owner_fee_percentage / 100;
    // credit the owner fee to the collection owner
    *(balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()) += owner_fee.clone();

    // transfer the money from the MP to the seller
    if transfer_fungible(
        &listing.payment_address,
        &(offer.price.clone() - owner_fee.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the seller
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, listing.payment_address))
            .or_default()) += offer.price.clone() - owner_fee.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            offer.payment_address.clone(),
            format!(
                "accept_offer fungible failed for user {} for buy id {}; transfer 2",
                listing.payment_address, buy_id
            ),
        ));

        offer.status = OfferStatus::Created;
        listing.status = ListingStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    // remove the sale offer and the bid that triggered the sale
    // all other bids still should remain valid
    offer.status = OfferStatus::Bought;
    mp.listings
        .remove(&(offer.nft_canister_id, offer.token_id.clone()));

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("acceptOffer")
                .details(vec![
                    ("token_id".into(), DetailValue::U64(offer.token_id.clone())),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(offer.nft_canister_id),
                    ),
                    ("buy_id".into(), DetailValue::U64(buy_id)),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(offer.price.clone()).unwrap()),
                    ),
                    (
                        "owner_fee".into(),
                        DetailValue::U64(convert_nat_to_u64(owner_fee).unwrap()),
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
pub async fn direct_buy(nft_canister_id: Principal, token_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let listing = mp
        .listings
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;

    // guarding against re-entrancy
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    if (!listing.is_direct_buyable) {
        return Err(MPApiError::Other(
            "Token is not available for direct buy".to_string(),
        ));
    }

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // check if the NFT is still hold by the seller
    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard.clone(),
    )
    .await?;
    if (token_owner.unwrap() != self_id) {
        return Err(MPApiError::InsufficientNonFungibleBalance);
    }

    // guarding agains reentrancy
    listing.status = ListingStatus::Selling;

    // get buyer balance
    let mut buyer_bal = balances()
        .balances
        .entry((collection.fungible_canister_id, caller))
        .or_default();

    // check if funds are deposited already, or call a transfer_from caller to MP canister
    if (buyer_bal.clone() < listing.price.clone()) {
        // insufficient balance, call transfer_from
        // Deposit tokens from the buyer to the MP contract
        if transfer_from_fungible(
            &caller,
            &self_id,
            &listing.price.clone(),
            &collection.fungible_canister_id,
            collection.fungible_canister_standard.clone(),
        )
        .await
        .is_err()
        {
            listing.status = ListingStatus::Created;
            return Err(MPApiError::TransferFungibleError);
        }

        // add balance to ledger
        *buyer_bal += listing.price.clone();
    }

    // transfer the nft from marketplace to the buyer
    if transfer_non_fungible(
        &caller,                                  // to
        &token_id,                                // nft id
        &nft_canister_id,                         // contract
        collection.nft_canister_standard.clone(), // nft type
    )
    .await
    .is_err()
    {
        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            caller.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 1",
        caller, nft_canister_id, token_id,
      ),
        ));

        listing.status = ListingStatus::Created;
        return Err(MPApiError::TransferNonFungibleError);
    }

    let owner_fee: Nat = listing.price.clone() * collection.owner_fee_percentage / 100;

    // todo: initiate transfer of fee to owner, if error fallback to credit in mp balance

    // credit the owner fee to the collection owner
    *(balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()) += owner_fee.clone();

    // transfer the funds from the MP to the seller, or
    if transfer_fungible(
        &listing.payment_address,
        &(listing.price.clone() - owner_fee.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the seller
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, listing.payment_address))
            .or_default()) += listing.price.clone() - owner_fee.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            caller.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 2",
        caller, nft_canister_id, token_id,
      ),
        ));
        //shouldn't return with error here, seller still gets funds through mp balance
    }

    // subtract amount from buyer balance
    *buyer_bal -= listing.price.clone();

    let price = listing.price.clone();

    // remove the sale offer and the bid that triggered the sale
    // all other bids still should remain valid
    mp.listings.remove(&(nft_canister_id, token_id.clone()));

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("directBuy")
                .details(vec![
                    ("token_id".into(), DetailValue::U64(token_id.clone())),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(nft_canister_id),
                    ),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(price).unwrap()),
                    ),
                    (
                        "owner_fee".into(),
                        DetailValue::U64(convert_nat_to_u64(owner_fee).unwrap()),
                    ),
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[query(name = "getAllListings")]
#[candid_method(query, rename = "getAllListings")]
pub async fn get_all_listings() -> Vec<((Principal, u64), Listing)> {
    marketplace()
        .listings
        .clone()
        .into_iter()
        .map(|offer| offer)
        .collect()
}

#[query(name = "getAllOffers")]
#[candid_method(query, rename = "getAllOffers")]
pub async fn get_all_offers(begin: u64, limit: u64) -> Vec<Offer> {
    let offers = &marketplace().offers;
    let result = offers[begin as usize..min((begin + limit) as usize, offers.len())].to_vec();

    result
}

/**
 * Deposit NFT
 * Canister should have allow access prior to deposit
 */
#[update(name = "depositNFT")]
#[candid_method(update, rename = "depositNFT")]
pub async fn deposit_nft(nft_canister_id: Principal, token_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // transfer nft caller -> marketplace
    if transfer_from_non_fungible(
        &caller,                                  // from
        &self_id,                                 // to
        &token_id.clone(),                        // nft id
        &nft_canister_id,                         // contract
        collection.nft_canister_standard.clone(), // nft type
    )
    .await
    .is_err()
    {
        return Err(MPApiError::TransferNonFungibleError);
    }

    // set token owner in balances ledger
    balances()
        .nft_balances
        .entry((nft_canister_id, token_id.clone()))
        .and_modify(|pid| *pid = caller)
        .or_insert(caller);

    Ok(())
}

#[update(name = "withdrawNFT")]
#[candid_method(update, rename = "withdrawNFT")]
pub async fn withdraw_nft(nft_canister_id: Principal, token_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let mut success = false;

    if let Some(owner) = balances()
        .nft_balances
        .get(&(nft_canister_id, token_id.clone()))
    {
        // marketplace owns this token, check if caller is the stored owner
        if (caller == *owner) {
            if transfer_non_fungible(
                &caller,
                &token_id.clone(),
                &nft_canister_id,
                collection.nft_canister_standard.clone(),
            )
            .await
            .is_err()
            {
                return Err(MPApiError::TransferNonFungibleError);
            }
            success = true;
        } else {
            return Err(MPApiError::InsufficientNonFungibleBalance);
        }
    } else {
        return Err(MPApiError::InsufficientNonFungibleBalance);
    }

    if (success) {
        balances()
            .nft_balances
            .remove(&(nft_canister_id, token_id.clone()));
    }

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

    if let Some(balance) = balances().balances.get_mut(&(fungible_canister_id, caller)) {
        let balance_to_send = balance.clone();
        *balance = Nat::from(0);
        if transfer_fungible(
            &caller,
            &balance_to_send,
            &fungible_canister_id,
            fungible_canister_standard.clone(),
        )
        .await
        .is_err()
        {
            *balance = balance_to_send;
        }

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id,
            caller.clone(),
            format!("withdraw failed for user {}", caller,),
        ));

        return Err(MPApiError::TransferFungibleError);
    }

    Ok(())
}

#[update(name = "cancelListing")]
#[candid_method(update, rename = "cancelListing")]
pub async fn cancel_listing(nft_canister_id: Principal, token_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();
    let listing = mp
        .listings
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;
    if (caller != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    mp.listings.remove(&(nft_canister_id, token_id.clone()));

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("cancelListing")
                .details(vec![
                    ("token_id".into(), DetailValue::U64(token_id)),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(nft_canister_id),
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
pub async fn cancel_offer(buy_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();

    let offer = mp
        .offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidOffer)?;

    if (caller != offer.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    offer.status = OfferStatus::Cancelled;

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("cancelOfferByBuyer")
                .details(vec![("buy_id".into(), DetailValue::U64(buy_id))])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "denyOffer")]
#[candid_method(update, rename = "denyOffer")]
pub async fn deny_offer(buy_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();

    let offer = mp
        .offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidOffer)?;
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    let listing = mp
        .listings
        .get_mut(&(offer.nft_canister_id, offer.token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;

    if (caller != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    offer.status = OfferStatus::Denied;

    // todo refund deposit logic

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("denyOffer")
                .details(vec![("buy_id".into(), DetailValue::U64(buy_id))])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "addCollection")]
#[candid_method(update, rename = "addCollection")]
fn add_collection(
    owner: Principal,
    owner_fee_percentage: u16,
    creation_time: u64,
    collection_name: String,
    nft_canister_id: Principal,
    nft_canister_standard: NFTStandard,
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) {
    // TODO: related to the init_data, which seems we can remove?
    // assert_eq!(ic::caller(), init_data().owner);

    collections().collections.insert(
        nft_canister_id,
        Collection::new(
            owner,
            owner_fee_percentage,
            creation_time,
            collection_name,
            nft_canister_id,
            nft_canister_standard,
            fungible_canister_id,
            fungible_canister_standard,
        ),
    );
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
