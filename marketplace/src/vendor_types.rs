use ic_kit::candid::{CandidType, Deserialize, Nat, Principal};

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

// BEGIN DIP-721v2 //

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

// END DIP-721v2 //

// BEGIN DIP-20 //

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

// END DIP-DIP20 //
