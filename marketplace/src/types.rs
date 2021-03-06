use crate::vendor_types::*;
use derive_new::*;
use ic_kit::{
    candid::{CandidType, Deserialize, Int, Nat},
    macros::*,
    Principal,
};
use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, VecDeque};

/* THREAD_LOCAL TYPES */

#[derive(CandidType, Clone, Deserialize, new)]
pub struct InitData {
    pub cap: Option<Principal>,
    pub owner: Principal,
    pub protocol_fee: Nat,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct Collection {
    pub owner: Principal,
    pub collection_fee: Nat,
    pub creation_time: u64,
    pub collection_name: String,
    pub nft_canister_id: Principal,
    pub nft_canister_standard: NFTStandard,
    pub fungible_canister_id: Principal,
    pub fungible_canister_standard: FungibleStandard,
    pub fungible_volume: Nat,
}

pub type Collections = HashMap<Principal, Collection>;

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct Balances {
    // (collection, user pid): value
    pub balances: HashMap<(Principal, Principal), Nat>,
    pub failed_tx_log_entries: Vec<TxLogEntry>,
}

#[derive(Default, CandidType, Clone, Deserialize, new)]
pub struct Marketplace {
    // collection { token: { listing } }
    pub listings: HashMap<Principal, HashMap<Nat, Listing>>,

    // collection: { token: { principal: offer } }
    pub offers: HashMap<Principal, HashMap<Nat, HashMap<Principal, Offer>>>,

    // user: (collection, token)
    pub user_offers: HashMap<Principal, HashMap<Principal, Vec<Nat>>>,
}

/* Data types */

#[derive(Clone, CandidType, Deserialize, Debug, new)]
pub struct Listing {
    pub price: Nat,
    pub seller: Principal,
    pub status: ListingStatus,
    pub created: u64,
    pub fee: Vec<(String, Principal, Nat)>,
}

impl Default for Listing {
    fn default() -> Self {
        Listing::new(
            Nat::from(0),
            Principal::anonymous(),
            ListingStatus::Uninitialized,
            0,
            Vec::new(),
        )
    }
}

#[derive(Clone, CandidType, Debug, Deserialize, PartialEq, new)]
pub struct Offer {
    pub nft_canister_id: Principal,
    pub token_id: Nat,
    pub price: Nat,
    pub buyer: Principal,
    pub token_owner: Principal,
    pub status: OfferStatus,
    pub created: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum MPApiError {
    InvalidOperator,
    InvalidOwner,
    TransferFromFungibleError(String),
    TransferFromNonFungibleError(String),

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

#[derive(Clone, CandidType, Deserialize, new)]
pub struct TxLogEntry {
    pub from: Principal,
    pub to: Principal,
    pub memo: String,
}

pub type MPApiResult = Result<(), MPApiError>;
pub type PrincipalResult = Result<Option<Principal>, MPApiError>;
pub type U64Result = Result<u64, MPApiError>;
pub type NatResult = Result<Nat, MPApiError>;

#[derive(Clone, CandidType, Deserialize)]
pub enum FungibleStandard {
    DIP20,
}

#[derive(Copy, Clone, CandidType, Deserialize)]
pub enum NFTStandard {
    DIP721v2,
    EXT,
}
