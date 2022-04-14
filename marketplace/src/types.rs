use crate::vendor_types::*;

use cap_sdk::{insert, Event, IndefiniteEvent, TypedEvent};
use ic_kit::{
    candid::{CandidType, Deserialize, Int, Nat},
    Principal,
};

use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, VecDeque};

use derive_new::*;

#[derive(Clone, CandidType, Debug, Deserialize, Eq, PartialEq)]
pub enum OfferStatus {
    Uninitialized,
    Created,
    Cancelled,
    Denied,
    Bought,
}

#[derive(Clone, CandidType, Deserialize, Debug, Eq, PartialEq)]
pub enum ListingStatus {
    Uninitialized,
    Created,
    Selling,
}

pub struct InitData {
    pub cap: Principal,
    pub owner: Principal,
}

#[derive(Clone, CandidType, Deserialize, Debug, new)]
pub struct Listing {
    pub direct_buy: bool,
    pub price: Nat,
    pub payment_address: Principal,
    pub status: ListingStatus,
}

impl Default for Listing {
    fn default() -> Self {
        Listing::new(
            false,
            Nat::from(0),
            Principal::anonymous(),
            ListingStatus::Uninitialized,
        )
    }
}

#[derive(Clone, CandidType, Debug, Deserialize, PartialEq, new)]
pub struct Offer {
    pub nft_canister_id: Principal,
    pub token_id: Nat,
    pub price: Nat,
    pub payment_address: Principal,
    pub status: OfferStatus,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct Collection {
    pub owner: Principal,
    pub owner_fee_percentage: Nat,
    pub creation_time: u64,
    pub collection_name: String,
    pub nft_canister_id: Principal,
    pub nft_canister_standard: NFTStandard,
    pub fungible_canister_id: Principal,
    pub fungible_canister_standard: FungibleStandard,
}

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct Collections {
    pub collections: HashMap<Principal, Collection>,
}

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct FungibleBalance {
    pub amount: Nat,
    pub locked: Nat,
}

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct Balances {
    // (collection, user pid): value
    pub balances: HashMap<(Principal, Principal), FungibleBalance>,
    pub nft_balances: HashMap<(Principal, Nat), Principal>,
    pub failed_tx_log_entries: Vec<TxLogEntry>,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct TxLogEntry {
    pub from: Principal,
    pub to: Principal,
    pub memo: String,
}

#[derive(Default)]
pub(crate) struct Marketplace {
    // (collection, token): listing
    pub listings: HashMap<(Principal, Nat), Listing>,

    // collection: { token: { principal: offer } }
    pub alt_offers: HashMap<Principal, HashMap<Nat, HashMap<Principal, Offer>>>,
    pub offers: Vec<Offer>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum MPApiError {
    TransferFungibleError,
    TransferNonFungibleError,
    InvalidListingStatus,
    InvalidOfferStatus,
    InvalidListing,
    InvalidOffer,
    InsufficientFungibleBalance,
    InsufficientFungibleAllowance,
    InsufficientNonFungibleBalance,
    Unauthorized,
    NoDeposit,
    CAPInsertionError,
    NonExistentCollection,
    Other(String),
}

pub type MPApiResult = Result<(), MPApiError>;
pub type PrincipalResult = Result<Option<Principal>, MPApiError>;
pub type U64Result = Result<u64, MPApiError>;
pub type NatResult = Result<Nat, MPApiError>;

#[derive(Clone, CandidType, Deserialize)]
pub enum FungibleStandard {
    DIP20,
}

impl FungibleStandard {
    pub fn to_string(&self) -> String {
        match self {
            FungibleStandard::DIP20 => "DIP20".to_string(),
        }
    }
}

#[derive(Copy, Clone, CandidType, Deserialize)]
pub enum NFTStandard {
    DIP721v2,
    EXT,
}

impl NFTStandard {
    pub fn to_string(&self) -> String {
        match self {
            NFTStandard::DIP721v2 => "DIP721v2".to_string(),
            NFTStandard::EXT => "EXT".to_string(),
        }
    }
}

#[derive(Default)]
pub struct CapQ {
    ie_records: VecDeque<IndefiniteEvent>,
}

impl CapQ {
    pub async fn insert_into_cap(&mut self, ie: IndefiniteEvent) -> TxReceipt {
        if let Some(failed_ie) = self.ie_records.pop_front() {
            let _ = self.insert_into_cap_priv(failed_ie).await;
        }
        self.insert_into_cap_priv(ie).await
    }

    async fn insert_into_cap_priv(&mut self, ie: IndefiniteEvent) -> TxReceipt {
        let insert_res = insert(ie.clone())
            .await
            .map(|tx_id| Nat::from(tx_id))
            .map_err(|_| TxError::Other);

        if insert_res.is_err() {
            &mut self.ie_records.push_back(ie.clone());
        }

        match insert_res {
            Ok(r) => return TxReceipt::Ok(r),
            Err(e) => return TxReceipt::Err(e),
        }
    }
}
