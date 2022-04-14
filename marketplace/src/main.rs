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

    if (direct_buy) {
        if (token_owner.unwrap() != self_id) {
            return Err(MPApiError::NoDeposit);
        }
    } else {
        if (token_owner.unwrap() != seller) {
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
        seller,
        ListingStatus::Created,
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
        &self_id,
        &buyer,
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
        .alt_offers
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
            )
        });

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
        .alt_offers
        .entry(nft_canister_id)
        .or_default()
        .entry(token_id.clone())
        .or_default();

    let offer = offers.get_mut(&buyer).ok_or(MPApiError::InvalidListing)?;
    // guarding against re-entrancy
    if offer.status != OfferStatus::Created {
        return Err(MPApiError::InvalidOfferStatus);
    }

    let listing = mp
        .listings
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;

    // check if nft is held by marketplace
    let nft_owner = balances()
        .nft_balances
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::NoDeposit)?;

    if (nft_owner.clone() != seller) {
        return Err(MPApiError::Unauthorized);
    }

    // guarding agains reentrancy
    listing.status = ListingStatus::Selling;

    // Withdraw

    // check if marketplace has allowance
    let allowance = allowance_fungible(
        &collection.fungible_canister_id,
        &self_id,
        &buyer,
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

    // withdraw funds from buyer wallet to mkp
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
    } else {
        // Successfully withdrawn from buyer wallet, transfer the funds from the MP to the seller, or fallback to balance.
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
    }

    // transfer the nft from marketplace to the buyer
    if transfer_non_fungible(
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        // fallback to balance
        *nft_owner = buyer;

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            seller.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 1",
        seller, nft_canister_id, token_id,
      ),
        ));
    }

    offer.status = OfferStatus::Bought;

    let price = offer.price.clone();

    // remove listing and offer
    mp.listings.remove(&(nft_canister_id, token_id.clone()));
    offers.remove(&buyer);

    // avoid keeping an empty object for a token if no more offers
    if (offers.len() == 0) {
        mp.alt_offers
            .entry(nft_canister_id)
            .or_default()
            .remove(&token_id.clone());
    }

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

    let listing = mp
        .listings
        .get_mut(&(nft_canister_id, token_id.clone()))
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

    // check if the NFT is owned by mp
    let _owner = owner_of_non_fungible(
        &nft_canister_id,
        &token_id,
        collection.nft_canister_standard,
    )
    .await?;
    if (_owner.unwrap() != self_id) {
        return Err(MPApiError::NoDeposit);
    }

    // check if nft is held by marketplace
    let nft_owner = balances()
        .nft_balances
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::NoDeposit)?;

    if (nft_owner.clone() != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    // guarding agains reentrancy
    listing.status = ListingStatus::Selling;

    // get buyer balance
    let mut buyer_bal = balances()
        .balances
        .entry((collection.fungible_canister_id, buyer))
        .or_default();

    // check if funds are deposited
    if (buyer_bal.amount.clone() < listing.price.clone()) {
        // insufficient balance
        listing.status = ListingStatus::Created;
        return Err(MPApiError::InsufficientFungibleBalance);
    }

    // transfer the nft from marketplace to the buyer
    if transfer_non_fungible(
        &buyer,                           // to
        &token_id,                        // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        // fallback to balance
        *nft_owner = buyer;

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buyer.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 1",
        buyer, nft_canister_id, token_id,
      ),
        ));
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
        // credit the funds to the seller
        balances()
            .balances
            .entry((collection.fungible_canister_id, listing.payment_address))
            .or_default()
            .amount += listing.price.clone() - owner_fee.clone();

        balances().failed_tx_log_entries.push(TxLogEntry::new(
            self_id.clone(),
            buyer.clone(),
            format!(
        "direct_buy non fungible failed for user {} for contract {} for token id {}; transfer 2",
        buyer, nft_canister_id, token_id,
      ),
        ));
        //shouldn't return with error here, seller still gets funds through mp balance
    }

    // subtract amount from buyer balance
    buyer_bal.amount -= listing.price.clone();

    let price = listing.price.clone();

    // remove listing
    mp.listings.remove(&(nft_canister_id, token_id.clone()));

    if (buyer_bal.amount.clone() == Nat::from(0)) {
        balances()
            .balances
            .remove(&(collection.fungible_canister_id, buyer));
    }

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
pub async fn get_all_listings() -> Vec<((Principal, Nat), Listing)> {
    marketplace()
        .listings
        .clone()
        .into_iter()
        .map(|offer| offer)
        .collect()
}

#[query(name = "getAllOffers")]
#[candid_method(query, rename = "getAllOffers")]
pub async fn get_all_offers() -> HashMap<Principal, HashMap<Nat, HashMap<Principal, Offer>>> {
    marketplace().alt_offers.clone()
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

/**
 * Deposit NFT
 * Canister should have allow access prior to deposit
 */
#[update(name = "depositNFT")]
#[candid_method(update, rename = "depositNFT")]
pub async fn deposit_nft(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
    let caller = ic::caller();
    let self_id = ic::id();
    let collection = collections()
        .collections
        .get(&nft_canister_id)
        .ok_or(MPApiError::NonExistentCollection)?;

    // transfer nft caller -> marketplace
    if transfer_from_non_fungible(
        &caller,                          // from
        &self_id,                         // to
        &token_id.clone(),                // nft id
        &nft_canister_id,                 // contract
        collection.nft_canister_standard, // nft type
    )
    .await
    .is_err()
    {
        return Err(MPApiError::TransferNonFungibleError);
    }

    // set token owner in balances ledger
    balances()
        .nft_balances
        .insert((nft_canister_id, token_id.clone()), caller);

    Ok(())
}

#[update(name = "withdrawNFT")]
#[candid_method(update, rename = "withdrawNFT")]
pub async fn withdraw_nft(nft_canister_id: Principal, token_id: Nat) -> MPApiResult {
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
        .clone()
    {
        // marketplace owns this token, check if caller is the stored owner
        if (caller == *owner) {
            if transfer_non_fungible(
                &caller,
                &token_id.clone(),
                &nft_canister_id,
                collection.nft_canister_standard,
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
        // remove balance
        balances()
            .nft_balances
            .remove(&(nft_canister_id, token_id.clone()));

        // remove listing if exists
        marketplace()
            .listings
            .remove(&(nft_canister_id, token_id.clone()));
    }

    Ok(())
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
    let listing = mp
        .listings
        .get_mut(&(nft_canister_id, token_id.clone()))
        .ok_or(MPApiError::InvalidListing)?;
    if (seller != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }
    if listing.status != ListingStatus::Created {
        return Err(MPApiError::InvalidListingStatus);
    }

    // todo: attempt auto withdraw

    let old_price = listing.price.clone();

    mp.listings.remove(&(nft_canister_id, token_id.clone()));

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

    let mut offers = mp
        .alt_offers
        .entry(nft_canister_id)
        .or_default()
        .entry(token_id.clone())
        .or_default();

    offers.remove(&buyer);

    if (offers.len() == 0) {
        mp.alt_offers
            .entry(nft_canister_id)
            .or_default()
            .remove(&token_id.clone());
    }

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(buyer)
                .operation("cancelOffer")
                .details(vec![])
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
    let seller = ic::caller();
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

    if (seller != listing.payment_address) {
        return Err(MPApiError::Unauthorized);
    }

    offer.status = OfferStatus::Denied;

    capq()
        .insert_into_cap(
            IndefiniteEventBuilder::new()
                .caller(seller)
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
