use ic_kit::candid::{CandidType, Deserialize, Int, Nat, Principal};
use std::collections::HashMap;

// BEGIN ServiceBalance //

#[derive(CandidType, Clone, Deserialize)]
pub enum GenericValue {
    BoolContent(bool),
    TextContent(String),
    BlobContent(Vec<u8>),
    Principal(Principal),
    Nat8Content(u8),
    Nat16Content(u16),
    Nat32Content(u32),
    Nat64Content(u64),
    NatContent(Nat),
    Int8Content(i8),
    Int16Content(i16),
    Int32Content(i32),
    Int64Content(i64),
    IntContent(Int),
    FloatContent(f64), // motoko only support f64
    NestedContent(Vec<(String, GenericValue)>),
}

#[derive(CandidType, Clone, Deserialize)]
pub struct BalanceMetadata {
    pub owner: Principal,
    pub contractId: Principal,
    pub standard: String,
    pub token_type: String,
    pub details: HashMap<String, GenericValue>,
}

// END ServiceBalance //

// BEGIN EXT //

pub type AccountIdentifier = String;
pub type TokenIdentifier = u64;
pub type Balance = Nat;
pub type Blob = Vec<u8>;
pub type Memo = Blob;
pub type SubAccount = Vec<u8>;

#[derive(CandidType, Debug, Deserialize, PartialEq)]
pub enum User {
    address(AccountIdentifier),
    principal(Principal),
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
pub struct TransferRequest {
    pub from: User,
    pub to: User,
    pub token: TokenIdentifier,
    pub amount: Balance,
    pub memo: Memo,
    pub notify: bool,
    pub subaccount: Option<SubAccount>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
pub enum TransferResponseErrors {
    Unauthorized(AccountIdentifier),
    InsufficientBalance,
    Rejected,
    InvalidToken(TokenIdentifier),
    CannotNotify(AccountIdentifier),
    Other(String),
}

pub type TransferResponse = Result<Balance, TransferResponseErrors>;

// END EXT //

// BEGIN DIP721v2 //

#[derive(CandidType, Debug, Deserialize)]
pub enum ApiError {
    Unauthorized,
    InvalidTokenId,
    ZeroAddress,
    Other,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum NftError {
    UnauthorizedOwner,
    UnauthorizedOperator,
    OwnerNotFound,
    OperatorNotFound,
    TokenNotFound,
    ExistedNFT,
    SelfApprove,
    SelfTransfer,
    TxNotFound,
    Other(String),
}

pub type TxReceiptDIP721v2 = Result<Nat, ApiError>;
pub type OwnerResult = Result<Principal, NftError>;

// END DIP721v2 //

// BEGIN DIP20 //

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
pub enum TxError {
    InsufficientAllowance,
    InsufficientBalance,
    ErrorOperationStyle,
    Unauthorized,
    LedgerTrap,
    ErrorTo,
    Other,
    BlockUsed,
    AmountTooSmall,
}
pub type TxReceipt = Result<Nat, TxError>;

// END DIP20 //
