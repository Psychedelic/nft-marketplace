use cap_sdk::CapEnv;
use ic_kit::{
    candid::{CandidType, Deserialize, Int, Nat, Principal},
    ic::{get_maybe, get_mut, stable_restore, stable_store, store},
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

#[pre_upgrade]
fn pre_upgrade() {
    let marketplace = marketplace(|marketplace| marketplace.clone());
    let collections = collections(|collections| collections.clone());
    let balances = balances(|balances| balances.clone());
    let init_data = init_data(|init_data| init_data.clone());
    stable_store((
        marketplace,
        collections,
        balances,
        init_data,
        cap_sdk::archive(),
    ))
    .unwrap();
}

// BEGIN POST_UPGRADE #1 //

#[derive(Clone, CandidType, Default, Deserialize)]
pub struct OldCollections {
    pub collections: HashMap<Principal, Collection>,
}

#[post_upgrade]
fn post_upgrade() {
    let (
        marketplace_stored,
        collections_stored,
        balances_stored,
        init_data_stored,
        cap_env_stored,
    ): (Marketplace, OldCollections, Balances, InitData, cap_sdk::Archive) = stable_restore().unwrap();
    marketplace_mut(|marketplace| {
        marketplace.listings = marketplace_stored.listings;
        marketplace.offers = marketplace_stored.offers;
        marketplace.user_offers = marketplace_stored.user_offers;
    });
    collections_mut(|collections| {
        collections.extend(collections_stored.collections);
    });
    balances_mut(|balances| {
        balances.balances = balances_stored.balances;
        balances.failed_tx_log_entries = balances_stored.failed_tx_log_entries;
    });
    init_data_mut(|init_data| {
        init_data.cap = init_data_stored.cap;
        init_data.owner = init_data_stored.owner;
        init_data.protocol_fee = init_data_stored.protocol_fee;
    });
    cap_sdk::from_archive(cap_env_stored);
}

// END POST_UPGRADE #1 //

// BEGIN POST_UPGRADE #2 //

// #[post_upgrade]
fn _post_upgrade() {
    let (
        marketplace_stored,
        collections_stored,
        balances_stored,
        init_data_stored,
        cap_env_stored,
    ): (Marketplace, Collections, Balances, InitData, cap_sdk::Archive) = stable_restore().unwrap();
    marketplace_mut(|marketplace| {
        marketplace.listings = marketplace_stored.listings;
        marketplace.offers = marketplace_stored.offers;
        marketplace.user_offers = marketplace_stored.user_offers;
    });
    collections_mut(|collections| {
        collections.extend(collections_stored);
    });
    balances_mut(|balances| {
        balances.balances = balances_stored.balances;
        balances.failed_tx_log_entries = balances_stored.failed_tx_log_entries;
    });
    init_data_mut(|init_data| {
        init_data.cap = init_data_stored.cap;
        init_data.owner = init_data_stored.owner;
        init_data.protocol_fee = init_data_stored.protocol_fee;
    });
    cap_sdk::from_archive(cap_env_stored);
}

// END POST_UPGRADE #2 //

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
