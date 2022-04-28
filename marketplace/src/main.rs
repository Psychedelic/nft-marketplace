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

// QUERY METHODS //

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

#[update(name = "makeListing")]
#[candid_method(update, rename = "makeListing")]
pub async fn make_listing(
    direct_buy: bool,
    nft_canister_id: Principal,
    token_id: Nat,
    price: Nat,
) -> MPApiResult {
    // Todo: if caller is a curator pid, handle fees
    let seller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    if (token_owner.unwrap() != seller) {
        return Err(MPApiError::Unauthorized);
    }

    let token_operator = operator_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?
    .ok_or(MPApiError::InvalidOperator)?;

    if (token_operator != self_id) {
        return Err(MPApiError::InvalidOperator);
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
        direct_buy.clone(),
        price.clone(),
        seller,
        ListingStatus::Created,
        ic::time(),
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
                    (
                        "direct_buy".into(),
                        match (direct_buy) {
                            true => DetailValue::True,
                            false => DetailValue::False,
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
pub async fn make_offer(nft_canister_id: Principal, token_id: Nat, price: Nat) -> MPApiResult {
    let buyer = ic::caller();
    let self_id = ic::id();
    let mut mp = marketplace();

    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    let buyer_balance = balances()
        .balances
        .entry((collection.fungible_canister_id, buyer))
        .or_default();

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &buyer,
        &self_id,
        collection.fungible_canister_standard.clone(),
    )
    .await;

    if allowance.is_err() {
        return Err(MPApiError::Other("Error calling allowance".to_string()));
    } else if allowance.ok().unwrap().clone() < price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check wallet balance for token
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await;

    if balance.is_err() {
        return Err(MPApiError::Other("Error calling balanceOf".to_string()));
    } else if balance.ok().unwrap().clone() < price.clone() {
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

    // guarding against re-entrancy
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    let listings = mp.listings.entry(nft_canister_id).or_default();

    let listing = listings
        .get_mut(&token_id.clone())
        .ok_or(MPApiError::InvalidListing)?;

    // check if the NFT is owned by the seller still
    let token_owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    if (token_owner.unwrap() != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    // check if mp is the operator still
    let token_operator = operator_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    if (token_operator.unwrap() != self_id) {
        return Err(MPApiError::InvalidOperator);
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
    .await;

    if allowance.is_err() {
        return Err(MPApiError::Other("Error calling allowance".to_string()));
    } else if allowance.ok().unwrap().clone() < offer.price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await;

    if balance.is_err() {
        return Err(MPApiError::Other("Error calling balanceOf".to_string()));
    } else if balance.ok().unwrap().clone() < offer.price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    let owner_fee: Nat =
        offer.price.clone() * collection.owner_fee_percentage.clone() / Nat::from(100);

    // auto deposit funds to mp from buyer
    if transfer_from_fungible(
        &buyer,
        &self_id,
        &offer.price.clone(),
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
        balances()
            .balances
            .entry((collection.fungible_canister_id, buyer))
            .or_default()
            .amount += listing.price.clone();

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

    // successfully transferred nft to buyer, release funds to seller
    if transfer_fungible(
        &seller,
        &(offer.price.clone() - owner_fee.clone()),
        &collection.fungible_canister_id,
        collection.fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        balances()
            .balances
            .entry((collection.fungible_canister_id, seller))
            .or_default()
            .amount += listing.price.clone() - owner_fee.clone();
    }

    // credit the owner fee to the collection owners balance
    balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()
        .amount += owner_fee.clone();

    offer.status = OfferStatus::Bought;

    let price = offer.price.clone();

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
                        DetailValue::U64(convert_nat_to_u64(price.clone()).unwrap()),
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

    if (!listing.direct_buy) {
        return Err(MPApiError::Other(
            "Token is not available for direct buy".to_string(),
        ));
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

    if (token_owner.unwrap() != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    // check if mp is the operator still
    let token_operator = operator_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;

    if (token_operator.unwrap() != self_id) {
        return Err(MPApiError::InvalidOperator);
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
    .await;

    if allowance.is_err() {
        return Err(MPApiError::Other("Error calling allowance".to_string()));
    } else if allowance.ok().unwrap().clone() < listing.price.clone() {
        return Err(MPApiError::InsufficientFungibleAllowance);
    }

    // check buyer wallet balance
    let balance = balance_of_fungible(
        &collection.fungible_canister_id,
        &buyer,
        collection.fungible_canister_standard.clone(),
    )
    .await;

    if balance.is_err() {
        return Err(MPApiError::Other("Error calling balanceOf".to_string()));
    } else if balance.ok().unwrap().clone() < listing.price.clone() {
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    let owner_fee: Nat =
        listing.price.clone() * collection.owner_fee_percentage.clone() / Nat::from(100);

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
            listing.payment_address.clone(),
            format!(
                "accept_offer failed for user {} for contract {} for token id {}; transfer 2",
                listing.payment_address, nft_canister_id, token_id,
            ),
        ));
        return Err(MPApiError::TransferFungibleError);
    }

    // Successfully auto deposited fungibles, transfer the nft from marketplace to the buyer
    if transfer_from_non_fungible(
        &listing.payment_address,         // from
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        // add deposited funds to buyer mp balance
        balances()
            .balances
            .entry((collection.fungible_canister_id, buyer))
            .or_default()
            .amount += listing.price.clone();

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

    let owner_fee: Nat =
        listing.price.clone() * collection.owner_fee_percentage.clone() / Nat::from(100);

    // todo: initiate transfer of fee to owner, if error fallback to credit in mp balance

    // credit the owner fee to the collection owner
    balances()
        .balances
        .entry((collection.fungible_canister_id, collection.owner))
        .or_default()
        .amount += owner_fee.clone();

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
        // fallback to sellers mp balance
        balances()
            .balances
            .entry((collection.fungible_canister_id, listing.payment_address))
            .or_default()
            .amount += listing.price.clone() - owner_fee.clone();
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
pub async fn get_all_listings(nft_canister_id: Principal) -> Vec<(Nat, Listing)> {
    let listings = marketplace()
        .listings
        .entry(nft_canister_id)
        .or_default()
        .clone();

    listings.into_iter().map(|offer| offer).collect()
}

#[query(name = "getAllOffers")]
#[candid_method(query, rename = "getAllOffers")]
pub async fn get_all_offers() -> HashMap<Principal, HashMap<Nat, HashMap<Principal, Offer>>> {
    marketplace().offers.clone()
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

#[query(name = getBuyerOffers)]
#[candid_method(query, rename = "getBuyerOffers")]
pub async fn get_buyer_offers(nft_canister_id: Principal, buyer: Principal) -> Vec<Offer> {
    let offers = marketplace()
        .offers
        .entry(nft_canister_id)
        .or_default()
        .clone();
    let token_list = marketplace()
        .user_offers
        .entry(buyer)
        .or_default()
        .entry(nft_canister_id)
        .or_default()
        .clone();

    let mut user_offers: Vec<Offer> = Vec::new();

    for token in token_list {
        user_offers.push(offers.get(&token).unwrap().get(&buyer).unwrap().clone())
    }

    user_offers
}

#[query(name = "getAllBalances")]
#[candid_method(query, rename = "getAllBalances")]
pub async fn get_all_balances() -> HashMap<(Principal, Principal), FungibleBalance> {
    let bals = &balances().balances;
    bals.clone()
}

#[query(name = "balanceOf")]
#[candid_method(query, rename = "balanceOf")]
pub async fn balance_of(pid: Principal) -> HashMap<Principal, FungibleBalance> {
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

#[query(name = "serviceBalanceOf")]
#[candid_method(query, rename = "serviceBalanceOf")]
pub async fn service_balance_of(pid: Principal) -> Vec<BalanceMetadata> {
    let caller = ic::caller();
    let collections: Vec<Collection> = collections().collections.values().cloned().collect();

    let mut total_balances: HashMap<Principal, BalanceMetadata> = HashMap::new();

    // index all registered collections
    for collection in collections {
        if !total_balances.contains_key(&collection.nft_canister_id) {
            let nft_bal: Vec<Nat> = balances()
                .nft_balances
                .clone()
                .into_iter()
                .filter_map(|((col, tok), owner)| {
                    if owner == pid {
                        return Some(tok.clone());
                    }
                    None
                })
                .collect();

            // todo status detail
            // todo reason detail

            for token in nft_bal {
                total_balances.insert(
                    collection.fungible_canister_id,
                    BalanceMetadata {
                        owner: pid,
                        contractId: collection.nft_canister_id,
                        standard: collection.nft_canister_standard.to_string(),
                        token_type: "NonFungible".to_string(),
                        details: HashMap::from([(
                            "token_id".to_string(),
                            GenericValue::TextContent(token.to_string()),
                        )]),
                    },
                );
            }
        }

        if !total_balances.contains_key(&collection.fungible_canister_id) {
            let fungible_bal = balances()
                .balances
                .entry((collection.fungible_canister_id, pid))
                .or_default();

            if fungible_bal.amount.clone() != Nat::from(0) {
                total_balances.insert(
                    collection.fungible_canister_id,
                    BalanceMetadata {
                        owner: pid,
                        contractId: collection.fungible_canister_id,
                        standard: collection.fungible_canister_standard.to_string(),
                        token_type: "Fungible".to_string(),
                        details: HashMap::from([(
                            "amount".to_string(),
                            GenericValue::NatContent(fungible_bal.amount.clone()),
                        )]),
                    },
                );
            }
        }
    }

    total_balances.values().cloned().collect()
}

#[update(name = "depositFungible")]
#[candid_method(update, rename = "depositFungible")]
pub async fn deposit_fungible(
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
    amount: Nat,
) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();

    // deposit funds
    if transfer_from_fungible(
        &caller,
        &self_id,
        &amount,
        &fungible_canister_id,
        fungible_canister_standard.clone(),
    )
    .await
    .is_err()
    {
        return Err(MPApiError::TransferFungibleError);
    }

    // deposit successful at this point, add balance to ledger
    balances()
        .balances
        .entry((fungible_canister_id, caller))
        .or_default()
        .amount += amount.clone();

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
        let balance_to_send = balance.amount.clone();
        if balance_to_send <= Nat::from(0) {
            return Err(MPApiError::Other(
                "Error: No unlocked tokens to withdraw".to_string(),
            ));
        }
        balance.amount -= balance_to_send.clone();
        if transfer_fungible(
            &caller,
            &balance_to_send,
            &fungible_canister_id,
            fungible_canister_standard.clone(),
        )
        .await
        .is_err()
        {
            balance.amount += balance_to_send.clone();
            balances().failed_tx_log_entries.push(TxLogEntry::new(
                self_id,
                caller.clone(),
                format!("withdraw failed for user {}", caller,),
            ));

            return Err(MPApiError::TransferFungibleError);
        }

        if (balance.amount == 0) {
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
        .get_mut(&token_id.clone())
        .ok_or(MPApiError::InvalidListing)?;
    if (seller != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    // todo: attempt auto withdraw

    let old_price = listing.price.clone();

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
                        DetailValue::U64(convert_nat_to_u64(old_price).unwrap()),
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

#[update(name = "addCollection")]
#[candid_method(update, rename = "addCollection")]
fn add_collection(
    owner: Principal,
    owner_fee_percentage: Nat,
    creation_time: u64,
    collection_name: String,
    nft_canister_id: Principal,
    nft_canister_standard: NFTStandard,
    fungible_canister_id: Principal,
    fungible_canister_standard: FungibleStandard,
) -> MPApiResult {
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

    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
