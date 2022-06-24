use cap_sdk::CapEnv;
use ic_kit::{
    candid::{CandidType, Deserialize, Int, Nat, Principal},
    ic::{stable_restore, stable_store, store},
    macros::*,
};
use num_bigint::Sign;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;

thread_local!(
    static MARKETPLACE: RefCell<Marketplace> = RefCell::new(Marketplace::new(
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ));
    static COLLECTIONS: RefCell<Collections> = RefCell::new(HashMap::new());
    static BALANCES: RefCell<Balances> = RefCell::new(Balances::new(HashMap::new(), Vec::new()));
    static INIT_DATA: RefCell<InitData> =
        RefCell::new(InitData::new(None, Principal::anonymous(), Nat::from(0)));
);

/// get mutable marketplace object from thread local
pub(crate) fn marketplace_mut<T, F: FnOnce(&mut Marketplace) -> T>(f: F) -> T {
    MARKETPLACE.with(|marketplace| f(&mut marketplace.borrow_mut()))
}

/// get marketplace object from thread local
pub(crate) fn marketplace<T, F: FnOnce(&Marketplace) -> T>(f: F) -> T {
    MARKETPLACE.with(|marketplace| f(&marketplace.borrow()))
}

pub(crate) fn collections_mut<T, F: FnOnce(&mut Collections) -> T>(f: F) -> T {
    COLLECTIONS.with(|collections| f(&mut collections.borrow_mut()))
}

pub(crate) fn collections<T, F: FnOnce(&Collections) -> T>(f: F) -> T {
    COLLECTIONS.with(|collections| f(&collections.borrow()))
}

pub(crate) fn balances_mut<T, F: FnOnce(&mut Balances) -> T>(f: F) -> T {
    BALANCES.with(|balances| f(&mut balances.borrow_mut()))
}

pub(crate) fn balances<T, F: FnOnce(&Balances) -> T>(f: F) -> T {
    BALANCES.with(|balances| f(&balances.borrow()))
}

pub(crate) fn init_data_mut<T, F: FnOnce(&mut InitData) -> T>(f: F) -> T {
    INIT_DATA.with(|init_data| f(&mut init_data.borrow_mut()))
}

pub(crate) fn init_data<T, F: FnOnce(&InitData) -> T>(f: F) -> T {
    INIT_DATA.with(|init_data| f(&init_data.borrow()))
}

pub(crate) fn remove_offer(nft_canister_id: &Principal, token_id: &Nat, user: &Principal) {
    marketplace_mut(|mp| {
        let mut offers = mp.offers.entry(*nft_canister_id).or_default();
        let mut token_offers = offers.entry(*token_id).or_default();

        token_offers.remove(&user);

        // save storage space
        if (token_offers.is_empty()) {
            offers.remove(&token_id.clone());
        }

        mp.user_offers
            .entry(*user)
            .or_default()
            .entry(*nft_canister_id)
            .and_modify(|tokens| {
                tokens.retain(|token| token != &token_id.clone());
            })
            .or_default();
    });
}

pub(crate) fn remove_listing(nft_canister_id: &Principal, token_id: &Nat) {
    marketplace_mut(|mp| {
        let listings = mp.listings.entry(*nft_canister_id).or_default();
        listings.remove(token_id);
    });
}

pub(crate) fn inc_volume(nft_canister_id: &Principal, amount: &Nat) {
    // update market cap for collection
    collections_mut(|collections| {
        collections
            .entry(*nft_canister_id)
            .and_modify(|collection_data| {
                collection_data.fungible_volume += amount.clone();
            });
    });
}

pub fn convert_nat_to_u64(num: Nat) -> Result<u64, String> {
    let u64_digits = num.0.to_u64_digits();

    match u64_digits.len() {
        0 => Ok(0),
        1 => Ok(u64_digits[0]),
        _ => Err("Nat -> Nat64 conversion failed".to_string()),
    }
}

pub fn convert_nat_to_u32(num: Nat) -> Result<u32, String> {
    let u32_digits = num.0.to_u32_digits();
    match u32_digits.len() {
        0 => Ok(0),
        1 => Ok(u32_digits[0]),
        _ => Err("Nat -> Nat32 conversion failed".to_string()),
    }
}

pub fn convert_int_to_u64(num: Int) -> Result<u64, String> {
    let u64_digits = num.0.to_u64_digits();

    if let Sign::Minus = u64_digits.0 {
        return Err("negative number cannot be converted to u64".to_string());
    }

    match u64_digits.1.len() {
        0 => Ok(0),
        1 => Ok(u64_digits.1[0]),
        _ => Err("Int -> Nat64 conversion failed".to_string()),
    }
}
