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
    token_id: &Nat,
    contract: &Principal,
    nft_type: NFTStandard,
) -> U64Result {
    match nft_type {
        NFTStandard::DIP721v2 => DIP721v2Proxy::transfer_from(from, to, token_id, contract).await,
        NFTStandard::EXT => EXTProxy::transfer_from(from, to, &convert_nat_to_u64(token_id.clone()).unwrap(), contract).await,
    }
}

pub async fn transfer_non_fungible(
    to: &Principal,
    token_id: &Nat,
    contract: &Principal,
    nft_type: NFTStandard,
) -> Result<Nat, MPApiError> {
    match nft_type {
        NFTStandard::DIP721v2 => DIP721v2Proxy::transfer(contract, to, token_id).await,
        NFTStandard::EXT => EXTProxy::transfer(to, token_id, contract).await,
    }
}

pub async fn owner_of_non_fungible(
    contract: &Principal,
    token_id: &Nat,
    nft_type: NFTStandard,
) -> PrincipalResult {
    match nft_type {
        NFTStandard::DIP721v2 => DIP721v2Proxy::owner_of(contract, token_id).await,
        NFTStandard::EXT => unimplemented!(),
    }
}

pub(crate) struct DIP721v2Proxy {}

impl DIP721v2Proxy {
    pub async fn transfer_from(
        from: &Principal,
        to: &Principal,
        token_id: &Nat,
        contract: &Principal,
    ) -> U64Result {
        let call_res: Result<(Result<Nat, NftError>,), (RejectionCode, String)> = ic::call(
            *contract,
            "transferFrom",
            (*from, *to, Nat::from(token_id.clone())),
        )
        .await;

        call_res
            .map_err(|_| MPApiError::Other("".to_string()))?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap())
    }

    pub async fn transfer(
        contract: &Principal,
        to: &Principal,
        token_id: &Nat,
    ) -> Result<Nat, MPApiError> {
        let call_res: Result<(Result<Nat, NftError>,), (RejectionCode, String)> =
            ic::call(*contract, "transfer", (*to, Nat::from(token_id.clone()))).await;

        let res = call_res
            .map_err(|_| MPApiError::Other("".to_string()))?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
            .map(|res| convert_nat_to_u64(res).unwrap());

        match &res {
            Ok(val) => Ok(Nat::from(val.clone())),
            _ => Err(MPApiError::TransferFungibleError),
        }
    }

    pub async fn owner_of(
        contract: &Principal,
        token_id: &Nat,
    ) -> Result<Option<Principal>, MPApiError> {
        let call_res: Result<(Result<Option<Principal>, NftError>,), (RejectionCode, String)> =
            ic::call(*contract, "ownerOf", (Nat::from(token_id.clone()),)).await;

        call_res
            .map_err(|_| MPApiError::Other("".to_string()))?
            .0
            .map_err(|_| MPApiError::TransferFungibleError)
    }
}

pub(crate) struct EXTProxy {}

impl EXTProxy {
    pub async fn transfer_from(
        from: &Principal,
        to: &Principal,
        token_id: &u64,
        contract: &Principal,
    ) -> U64Result {
        let call_res: Result<(TxReceiptDIP721v2,), (RejectionCode, String)> = ic::call(
            *contract,
            "transfer",
            (TransferRequest {
                from: User::principal(from.clone()),
                to: User::principal(to.clone()),
                token: token_id.clone(),
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

    pub async fn transfer(
        to: &Principal,
        token_id: &Nat,
        contract: &Principal,
    ) -> Result<Nat, MPApiError> {
        // // let res = DIP721v2Proxy::transfer_from(&ic::caller(), to, token_id.clone(), contract).await;

        // match res {
        //     Ok(val) => Ok(Nat::from(val)),
        //     _ => Err(MPApiError::TransferFungibleError),
        // }
        Ok(Nat::from(0))
    }
}
