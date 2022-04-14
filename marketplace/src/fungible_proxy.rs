use crate::types::*;
use crate::utils::convert_nat_to_u64;
use crate::vendor_types::*;

use ic_kit::{
    candid::{Nat, Principal},
    ic, RejectionCode,
};

pub struct FungibleProxy {}

// dynamic dispatch through trait objects is not implemented in rust for
// async functions, so we do the dispatch manually
pub async fn transfer_from_fungible(
    from: &Principal,
    to: &Principal,
    amount: &Nat,
    contract: &Principal,
    fungible_canister_standard: FungibleStandard,
) -> U64Result {
    match fungible_canister_standard {
        FungibleStandard::DIP20 => Dip20Proxy::transfer_from(from, to, amount, contract).await,
    }
}
pub async fn transfer_fungible(
    to: &Principal,
    amount: &Nat,
    contract: &Principal,
    fungible_canister_standard: FungibleStandard,
) -> U64Result {
    match fungible_canister_standard {
        FungibleStandard::DIP20 => Dip20Proxy::transfer(to, amount, contract).await,
    }
}

pub async fn balance_of_fungible(
    contract: &Principal,
    owner: &Principal,
    fungible_canister_standard: FungibleStandard,
) -> NatResult {
    match fungible_canister_standard {
        FungibleStandard::DIP20 => Dip20Proxy::balance_of(contract, owner).await,
    }
}

pub async fn allowance_fungible(
    contract: &Principal,
    owner: &Principal,
    spender: &Principal,
    fungible_canister_standard: FungibleStandard,
) -> NatResult {
    match fungible_canister_standard {
        FungibleStandard::DIP20 => Dip20Proxy::allowance(contract, owner, spender).await,
    }
}

pub(crate) struct Dip20Proxy {}

impl Dip20Proxy {
    pub async fn transfer_from(
        from: &Principal,
        to: &Principal,
        amount: &Nat,
        contract: &Principal,
    ) -> U64Result {
        let call_res: Result<(TxReceipt,), (RejectionCode, String)> =
            ic::call(*contract, "transferFrom", (*from, *to, amount.clone())).await;

        call_res
            .map_err(|_| MPApiError::TransferFungibleError)?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap())
    }

    pub async fn transfer(to: &Principal, amount: &Nat, contract: &Principal) -> U64Result {
        let call_res: Result<(TxReceipt,), (RejectionCode, String)> =
            ic::call(*contract, "transfer", (*to, amount.clone())).await;
        call_res
            .map_err(|_| MPApiError::TransferFungibleError)?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap())
    }

    pub async fn balance_of(contract: &Principal, owner: &Principal) -> NatResult {
        let call_res: Result<(Nat,), (RejectionCode, String)> =
            ic::call(*contract, "balanceOf", (*owner,)).await;
        call_res
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| res.0)
    }

    pub async fn allowance(
        contract: &Principal,
        owner: &Principal,
        spender: &Principal,
    ) -> NatResult {
        let call_res: Result<(Nat,), (RejectionCode, String)> =
            ic::call(*contract, "allowance", (*owner, *spender)).await;

        call_res
            .map_err(|_| MPApiError::InsufficientFungibleAllowance)
            .map(|res| res.0)
    }
}
