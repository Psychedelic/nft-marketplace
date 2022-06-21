use crate::*;
use ic_kit::{
  ic::{stable_restore, stable_store},
  macros::*,
};

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
