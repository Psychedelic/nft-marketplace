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
#[update(name = "listForSale")]
#[candid_method(update, rename = "listForSale")]
pub async fn list_for_sale(nft_canister_id: Principal, token_id: u64, price: Nat) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let token_owner =
        owner_of_non_fungible(&nft_canister_id, &token_id, collection.nft_type.clone()).await?;

    if (caller != token_owner.unwrap()) {
        return Err(MPApiError::Unauthorized);
    }

    let mut mp = marketplace();

    let mut sale_offer = mp
        .sale_offers
        .entry((nft_canister_id, token_id.clone()))
        .or_default();

    if (sale_offer.status == SaleOfferStatus::Selling) {
        return Err(MPApiError::InvalidSaleOfferStatus);
    }

    *sale_offer = SaleOffer::new(true, price.clone(), caller, SaleOfferStatus::Created);

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("makeSaleOffer")
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
                    ("payment_address".into(), DetailValue::Principal(caller)),
                ])
                .build()
                .unwrap(),
        )
        .await
        .map_err(|_| MPApiError::CAPInsertionError)?;

    Ok(())
}

#[update(name = "makeBuyOffer")]
#[candid_method(update, rename = "makeBuyOffer")]
pub async fn make_buy_offer(nft_canister_id: Principal, token_id: u64, price: Nat) -> U64Result {
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
        collection.fungible_canister_type.clone(),
    )
    .await?;

    if (fungible_balance < price) {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    mp.buy_offers.push(BuyOffer::new(
        nft_canister_id,
        token_id.clone(),
        price.clone(),
        caller,
        BuyOfferStatus::Created,
    ));
    let buy_id = mp.buy_offers.len() as u64;

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("makeBuyOffer")
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

    let buy_offer = mp
        .buy_offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidBuyOffer)?;

    // guarding against re-entrancy
    if buy_offer.status != BuyOfferStatus::Created {
        return Err(MPApiError::InvalidBuyOfferStatus);
    }

    let sale_offer = mp
        .sale_offers
        .get_mut(&(buy_offer.nft_canister_id, buy_offer.token_id.clone()))
        .ok_or(MPApiError::InvalidSaleOffer)?;

    // guarding against re-entrancy
    if sale_offer.status != SaleOfferStatus::Created {
        return Err(MPApiError::InvalidSaleOfferStatus);
    }

    // only the seller can accept the bid
    if (sale_offer.payment_address != caller) {
        return Err(MPApiError::Unauthorized);
    }

    let collection = collections()
        .collections
        .get(&buy_offer.nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // check if the NFT is still hold by the seller
    let token_owner = owner_of_non_fungible(
        &buy_offer.nft_canister_id,
        &buy_offer.token_id,
        collection.nft_type.clone(),
    )
    .await?;

    if (sale_offer.payment_address != token_owner.unwrap()) {
        mp.sale_offers
            .remove(&(buy_offer.nft_canister_id, buy_offer.token_id.clone()));
        return Err(MPApiError::InsufficientNonFungibleBalance);
    }

    // guarding agains reentrancy
    buy_offer.status = BuyOfferStatus::Bought;
    sale_offer.status = SaleOfferStatus::Selling;

    // transfer the money from the buyer to the MP contract
    if transfer_from_fungible(
        &buy_offer.payment_address,
        &self_id,
        &buy_offer.price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_type.clone(),
    )
    .await
    .is_err()
    {
        buy_offer.status = BuyOfferStatus::Created;
        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    // transfer the nft from the seller to the buyer
    if transfer_from_non_fungible(
        &sale_offer.payment_address,
        &buy_offer.payment_address,
        &buy_offer.token_id,
        &buy_offer.nft_canister_id,
        collection.nft_type.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the buyer, as he never received the NFT
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, buy_offer.payment_address))
            .or_default()) += buy_offer.price.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buy_offer.payment_address.clone(),
            format!(
                "accept_buy_offer non fungible failed for user {} for buy id {}; transfer 1",
                buy_offer.payment_address, buy_id
            ),
        ));

        buy_offer.status = BuyOfferStatus::Created;
        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferNonFungibleError);
    }

    let owner_fee: Nat = buy_offer.price.clone() * collection.owner_fee_percentage / 100;
    // credit the owner fee to the collection owner
    *(balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()) += owner_fee.clone();

    // transfer the money from the MP to the seller
    if transfer_fungible(
        &sale_offer.payment_address,
        &(buy_offer.price.clone() - owner_fee.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_type.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the seller
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, sale_offer.payment_address))
            .or_default()) += buy_offer.price.clone() - owner_fee.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buy_offer.payment_address.clone(),
            format!(
                "accept_buy_offer fungible failed for user {} for buy id {}; transfer 2",
                sale_offer.payment_address, buy_id
            ),
        ));

        buy_offer.status = BuyOfferStatus::Created;
        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    // remove the sale offer and the bid that triggered the sale
    // all other bids still should remain valid
    buy_offer.status = BuyOfferStatus::Bought;
    mp.sale_offers
        .remove(&(buy_offer.nft_canister_id, buy_offer.token_id.clone()));

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("acceptBuyOffer")
                .details(vec![
                    (
                        "token_id".into(),
                        DetailValue::U64(buy_offer.token_id.clone()),
                    ),
                    (
                        "nft_canister_id".into(),
                        DetailValue::Principal(buy_offer.nft_canister_id),
                    ),
                    ("buy_id".into(), DetailValue::U64(buy_id)),
                    (
                        "price".into(),
                        DetailValue::U64(convert_nat_to_u64(buy_offer.price.clone()).unwrap()),
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

    let sale_offer = mp
        .sale_offers
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidSaleOffer)?;

    // guarding against re-entrancy
    if sale_offer.status != SaleOfferStatus::Created {
        return Err(MPApiError::InvalidSaleOfferStatus);
    }

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // check if the NFT is still hold by the seller
    let token_owner =
        owner_of_non_fungible(&nft_canister_id, &token_id, collection.nft_type.clone()).await?;
    if (sale_offer.payment_address != token_owner.unwrap()) {
        mp.sale_offers.remove(&(nft_canister_id, token_id.clone()));
        return Err(MPApiError::InsufficientNonFungibleBalance);
    }

    // guarding agains reentrancy
    sale_offer.status = SaleOfferStatus::Selling;

    // transfer the money from the buyer to the MP contract
    if transfer_from_fungible(
        &caller,
        &self_id,
        &sale_offer.price.clone(),
        &collection.fungible_canister_id,
        collection.fungible_canister_type.clone(),
    )
    .await
    .is_err()
    {
        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    // transfer the nft from the seller to the buyer
    if transfer_from_non_fungible(
        &sale_offer.payment_address, // from
        &caller,                     // to
        &token_id,                   // nft id
        &nft_canister_id,            // contract
        collection.nft_type.clone(), // nft type
    )
    .await
    .is_err()
    {
        // credit the bid price to the buyer, as he never received the NFT
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, caller))
            .or_default()) += sale_offer.price.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            caller.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 1",
        caller, nft_canister_id, token_id,
      ),
        ));

        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferNonFungibleError);
    }

    let owner_fee: Nat = sale_offer.price.clone() * collection.owner_fee_percentage / 100;
    // credit the owner fee to the collection owner
    *(balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()) += owner_fee.clone();

    // transfer the money from the MP to the seller
    if transfer_fungible(
        &sale_offer.payment_address,
        &(sale_offer.price.clone() - owner_fee.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_type.clone(),
    )
    .await
    .is_err()
    {
        // credit the bid price to the seller
        *(balances()
            .balances
            .entry((collection.fungible_canister_id, sale_offer.payment_address))
            .or_default()) += sale_offer.price.clone() - owner_fee.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            caller.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 2",
        caller, nft_canister_id, token_id,
      ),
        ));

        sale_offer.status = SaleOfferStatus::Created;
        return Err(MPApiError::TransferFungibleError);
    }

    let price = sale_offer.price.clone();

    // remove the sale offer and the bid that triggered the sale
    // all other bids still should remain valid
    mp.sale_offers.remove(&(nft_canister_id, token_id.clone()));

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
pub async fn get_all_listings() -> Vec<((Principal, u64), SaleOffer)> {
    marketplace()
        .sale_offers
        .clone()
        .into_iter()
        .map(|offer| offer)
        .collect()
}

#[query(name = "getAllBuyOffers")]
#[candid_method(query, rename = "getAllBuyOffers")]
pub async fn get_all_buy_offers(begin: u64, limit: u64) -> Vec<BuyOffer> {
    let buy_offers = &marketplace().buy_offers;
    let result =
        buy_offers[begin as usize..min((begin + limit) as usize, buy_offers.len())].to_vec();

    result
}

#[update(name = "withdrawFungible")]
#[candid_method(update, rename = "withdrawFungible")]
pub async fn withdraw_fungible(
    fungible_canister_id: Principal,
    fungible_canister_type: FungibleTokenType,
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
            fungible_canister_type.clone(),
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

#[update(name = "cancelListingBySeller")]
#[candid_method(update, rename = "cancelListingBySeller")]
pub async fn cancel_listing_by_seller(nft_canister_id: Principal, token_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();
    let sale_offer = mp
        .sale_offers
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidSaleOffer)?;
    if (caller != sale_offer.payment_address) {
        return Err(MPApiError::Unauthorized);
    }
    if sale_offer.status != SaleOfferStatus::Created {
        return Err(MPApiError::InvalidSaleOfferStatus);
    }

    mp.sale_offers.remove(&(nft_canister_id, token_id.clone()));

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("cancelListingBySeller")
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

#[update(name = "cancelOfferByBuyer")]
#[candid_method(update, rename = "cancelOfferByBuyer")]
pub async fn cancel_offer_by_buyer(buy_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();

    let buy_offer = mp
        .buy_offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidBuyOffer)?;

    if (caller != buy_offer.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    if buy_offer.status != BuyOfferStatus::Created {
        return Err(MPApiError::InvalidBuyOfferStatus);
    }

    buy_offer.status = BuyOfferStatus::CancelledByBuyer;

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

#[update(name = "cancelOfferBySeller")]
#[candid_method(update, rename = "cancelOfferBySeller")]
pub async fn cancel_offer_by_seller(buy_id: u64) -> MPApiResult {
    let caller = ic::caller();
    let mut mp = marketplace();

    let buy_offer = mp
        .buy_offers
        .get_mut(buy_id as usize)
        .ok_or(MPApiError::InvalidBuyOffer)?;
    if buy_offer.status != BuyOfferStatus::Created {
        return Err(MPApiError::InvalidBuyOfferStatus);
    }

    let sale_offer = mp
        .sale_offers
        .get_mut(&(buy_offer.nft_canister_id, buy_offer.token_id.clone()))
        .ok_or(MPApiError::InvalidSaleOffer)?;

    if (caller != sale_offer.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    buy_offer.status = BuyOfferStatus::CancelledBySeller;

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(caller)
                .operation("cancelOfferBySeller")
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
    nft_type: NonFungibleTokenType,
    fungible_canister_id: Principal,
    fungible_canister_type: FungibleTokenType,
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
            nft_type,
            fungible_canister_id,
            fungible_canister_type,
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
