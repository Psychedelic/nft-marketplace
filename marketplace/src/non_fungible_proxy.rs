use crate::types::*;
use crate::utils::convert_nat_to_u64;
use crate::vendor_types::*;

use ic_kit::{
    candid::{Nat, Principal},
    ic, RejectionCode,
};

// dynamic dispatch through trait objects is not implemented in rust for
// async functions, so we do the dispatch manually
pub async fn transfer_from_non_fungible(
    from: &Principal,
    to: &Principal,
    nft_id: &u64,
    contract: &Principal,
    non_fungible_token_type: NonFungibleTokenType,
) -> U64Result {
    match non_fungible_token_type {
        NonFungibleTokenType::DIP721 => {
            Dip721Proxy::transfer_from(from, to, nft_id, contract).await
        }
        NonFungibleTokenType::EXT => EXTProxy::transfer_from(from, to, nft_id, contract).await,
    }
}

pub async fn transfer_non_fungible(
    to: &Principal,
    nft_id: &u64,
    contract: &Principal,
    non_fungible_token_type: NonFungibleTokenType,
) -> U64Result {
    match non_fungible_token_type {
        NonFungibleTokenType::DIP721 => Dip721Proxy::transfer(to, nft_id, contract).await,
        NonFungibleTokenType::EXT => EXTProxy::transfer(to, nft_id, contract).await,
    }
}

pub async fn owner_of_non_fungible(
    contract: &Principal,
    token_id: &u64,
    non_fungible_token_type: NonFungibleTokenType,
) -> PrincipalResult {
    match non_fungible_token_type {
        NonFungibleTokenType::DIP721 => Dip721Proxy::owner_of(contract, token_id).await,
        NonFungibleTokenType::EXT => unimplemented!(),
    }
}

pub(crate) struct Dip721Proxy {}

impl Dip721Proxy {
    pub async fn transfer_from(
        from: &Principal,
        to: &Principal,
        token_id: &u64,
        contract: &Principal,
    ) -> U64Result {
        let call_res: Result<(TxReceiptDIP721,), (RejectionCode, String)> = ic::call(
            *contract,
            "transferFromDip721",
            (*from, *to, token_id),
        )
        .await;
        call_res
            .map_err(|_| MPApiError::TransferFungibleError)?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap())
    }

    pub async fn transfer(to: &Principal, nft_id: &u64, contract: &Principal) -> U64Result {
        Dip721Proxy::transfer_from(&ic::caller(), to, nft_id, contract).await
    }

    pub async fn owner_of(contract: &Principal, token_id: &u64) -> PrincipalResult {
        let call_res: Result<(OwnerResult,), (RejectionCode, String)> = ic::call(
            *contract,
            "ownerOfDip721",
            (token_id,),
        )
        .await;

        match &call_res {
            Ok(res) => {
                match &res.0 {
                    Ok(principal) => {
                        ic_cdk::println!("[debug] owner_of -> principal -> {:?}", principal.to_string());

                        return Ok(*principal);
                    },
                    _ => ic_cdk::println!("[debug] owner_of -> 1"),
                }
            },
            _ => ic_cdk::println!("[debug] owner_of -> 2"),
        };

        call_res
            .map_err(|_| MPApiError::TransferFungibleError)?
            .0
            .map_err(|_| MPApiError::TransferFungibleError) 
    }
}

pub(crate) struct EXTProxy {}

impl EXTProxy {
    pub async fn transfer_from(
        from: &Principal,
        to: &Principal,
        nft_id: &u64,
        contract: &Principal,
    ) -> U64Result {
        let call_res: Result<(TxReceiptDIP721,), (RejectionCode, String)> = ic::call(
            *contract,
            "transfer",
            (TransferRequest {
                from: User::principal(from.clone()),
                to: User::principal(to.clone()),
                token: nft_id.clone(),
                amount: Nat::from(1),
                memo: vec![],
                notify: false,
                subaccount: None,
            },),
        )
        .await;
        call_res
            .map_err(|_| MPApiError::TransferFungibleError)?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap())
    }

    pub async fn transfer(to: &Principal, nft_id: &u64, contract: &Principal) -> U64Result {
        Dip721Proxy::transfer_from(&ic::caller(), to, nft_id, contract).await
    }
}
