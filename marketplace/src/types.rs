use crate::vendor_types::*;

use cap_sdk::{insert, Event, IndefiniteEvent, TypedEvent};
use ic_kit::{
    candid::{CandidType, Deserialize, Nat},
    Principal,
};

use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, VecDeque};

use derive_new::*;

#[derive(Clone, CandidType, Debug, Deserialize, Eq, PartialEq)]
pub enum BuyOfferStatus {
    Uninitialized,
    Created,
    CancelledByBuyer,
    CancelledBySeller,
    Bought,
}

#[derive(Clone, CandidType, Deserialize, Eq, PartialEq)]
pub enum SaleOfferStatus {
    Uninitialized,
    Created,
    Selling,
}

pub struct InitData {
    pub cap: Principal,
    pub owner: Principal,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct SaleOffer {
    pub is_direct_buyable: bool,
    pub list_price: Nat,
    pub payment_address: Principal,
    pub status: SaleOfferStatus,
}

impl Default for SaleOffer {
    fn default() -> Self {
        SaleOffer::new(
            false,
            Nat::from(0),
            Principal::anonymous(),
            SaleOfferStatus::Uninitialized,
        )
    }
}

#[derive(Clone, CandidType, Debug, Deserialize, new)]
pub struct BuyOffer {
    pub non_fungible_contract_address: Principal,
    pub token_id: u64,
    pub price: Nat,
    pub payment_address: Principal,
    pub status: BuyOfferStatus,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct Collection {
    pub owner: Principal,
    pub owner_fee_percentage: u16,
    pub creation_time: u64,
    pub collection_name: String,
    pub non_fungible_contract_address: Principal,
    pub non_fungible_token_type: NonFungibleTokenType,
    pub fungible_contract_address: Principal,
    pub fungible_token_type: FungibleTokenType,
}

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct Collections {
    pub collections: HashMap<Principal, Collection>,
}

#[derive(Clone, CandidType, Default, Deserialize, new)]
pub struct Balances {
    pub balances: HashMap<(Principal, Principal), Nat>,
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
    pub sale_offers: HashMap<(Principal, u64), SaleOffer>,
    pub buy_offers: Vec<BuyOffer>,
}

#[derive(CandidType, Deserialize)]
pub enum MPApiError {
    TransferFungibleError,
    TransferNonFungibleError,
    InvalidSaleOfferStatus,
    InvalidBuyOfferStatus,
    InvalidSaleOffer,
    InvalidBuyOffer,
    InsufficientFungibleBalance,
    InsufficientNonFungibleBalance,
    Unauthorized,
    CAPInsertionError,
    NonExistentCollection,
    Other,
}

pub type MPApiResult = Result<(), MPApiError>;
pub type PrincipalResult = Result<Principal, MPApiError>;
pub type U64Result = Result<u64, MPApiError>;
pub type NatResult = Result<Nat, MPApiError>;

#[derive(Clone, CandidType, Deserialize)]
pub enum FungibleTokenType {
    DIP20,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum NonFungibleTokenType {
    DIP721,
    EXT,
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
