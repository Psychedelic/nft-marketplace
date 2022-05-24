use ic_kit::{
    macros::*,
    candid::{Int, Nat},
    ic::{stable_store, stable_restore, store, get_maybe, get_mut},
};
use cap_sdk::CapEnv;
use num_bigint::Sign;

use crate::types::*;

pub(crate) fn marketplace() -> &'static mut Marketplace {
    get_mut::<Marketplace>()
}

pub(crate) fn collections() -> &'static mut Collections {
    get_mut::<Collections>()
}
pub(crate) fn balances() -> &'static mut Balances {
    get_mut::<Balances>()
}

pub(crate) fn init_data() -> &'static InitData {
    get_maybe::<InitData>().unwrap()
}

#[pre_upgrade]
fn pre_upgrade() {
    let marketplace = marketplace();
    let collections = collections();
    let balances = balances();
    let init_data = init_data();
    stable_store((
        marketplace,
        collections,
        balances,
        init_data,
        CapEnv::to_archive(),
    ))
    .unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (
        marketplace_stored,
        collections_stored,
        balances_stored,
        init_data_stored,
        cap_env_stored,
    ): (Marketplace, Collections, Balances, InitData, CapEnv) = stable_restore().unwrap();
    store(marketplace_stored);
    store(collections_stored);
    store(balances_stored);
    store(init_data_stored);
    CapEnv::load_from_archive(cap_env_stored);
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
