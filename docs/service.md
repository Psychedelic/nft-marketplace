# Marketplace Service Documentation

The marketplace works on a deposit/withdraw mechanism to lock funds and nfts during transactions to ensure safe transfer.
On the occasion that a transfer error happens, all tokens are still available through a withdraw method.

## DID Interface

The marketplace provides the following service interface:

```
service : (principal, principal) -> {
  addCollection : (
      principal,
      nat16,
      nat64,
      text,
      principal,
      NFTStandard,
      principal,
      FungibleStandard,
    ) -> (Result);

  makeListing : (bool, principal, nat64, nat) -> (Result);
  cancelListing : (principal, nat64) -> (Result);
  getAllListings : () -> (
      vec record { record { principal; nat64 }; Listing },
    ) query;

  directBuy : (principal, nat64) -> (Result);

  makeOffer : (principal, nat64, nat) -> (Result);
  cancelOffer : (principal, nat64) -> (Result);
  denyOffer : (nat64) -> (Result);
  acceptOffer : (principal, nat64, principal) -> (Result);
  getAllOffers : () -> (
      vec record {
        principal;
        vec record { nat64; vec record { principal; Offer } };
      },
    ) query;

  depositFungible : (principal, FungibleStandard, nat) -> (Result);
  depositNFT : (principal, nat64) -> (Result);
  withdrawFungible : (principal, FungibleStandard) -> (Result);
  withdrawNFT : (principal, nat64) -> (Result);

  serviceBalanceOf : (principal) -> (vec BalanceMetadata) query;
  balanceOf : (principal) -> (vec record { principal; FungibleBalance }) query;
  getAllBalances : () -> (
      vec record { record { principal; principal }; FungibleBalance },
    ) query;
}
```

## Service Balance Inferface

There is a simple interface for querying for assets held by the service.

```
// Method //

serviceBalanceOf : (principal) -> (vec BalanceMetadata) query;

// Types //

type GenericValue = variant {
  Nat64Content : nat64;
  Nat32Content : nat32;
  BoolContent : bool;
  Nat8Content : nat8;
  Int64Content : int64;
  IntContent : int;
  NatContent : nat;
  Nat16Content : nat16;
  Int32Content : int32;
  Int8Content : int8;
  FloatContent : float64;
  Int16Content : int16;
  BlobContent : vec nat8;
  NestedContent : vec record { text; GenericValue };
  Principal : principal;
  TextContent : text;
};

type BalanceMetadata = record {
  owner : principal;
  details : vec record { text; GenericValue };
  token_type : text;
  standard : text;
  contractId : principal;
};
```

### Details Vec

For details, there are slightly different values depending on if `token_type` = `Fungible` or `NonFungible`

#### Fungible:

```
details: vec {
  amount: GenericValue::Nat
}
```

#### NonFungible:

```
details: vec {
  token_id: GenericValue::Nat
}
```

## Direct Buy Flow:

### Listing NFT for direct buy -> `makeListing`

to create a listing that is available for direct buy, the nft must be deposited before the listing will go though.

```

call approve on crowns canister for marketplace on token
call depositNFT on marketplace canister
call makeListing with direct_buy = true

```

### Direct Buy NFT -> `directBuy`

to direct buy a listing, you must deposit funds before the direct buy will go through.

```

call approve on wicp for marketplace to access x tokens
call depositFungable on marketplace canister
call directBuy on marketplace canister for nft

```

## Offer Flow:

- a nft can be listed without a deposit if direct buy is disabled. A user can accept offers regardless of listing status (unlisted, listed for direct buy, listed for offers). When a seller decides to accept an offer, the nft must be deposited.

### Listing NFT for offers -> `makeListing`

To create a listing that is only available for offers, you only have to be the current owner of a given nft. No deposit required at the time of listing

```

call makeListing with direct_buy = false

```

### Making an offer -> `makeOffer`

- to make an offer, you must set a proper allowance for marketplace for a users funds. This can be a one time, really high amount, or with each offer.
- If the allowance is set with each offer, the amount allowed for marketplace should always be equal or more than the total amount offered for a user.
- A user must also have at least the amount offered held in their wallet, this is double checked in the backend, but FE should also track a users' wallets' balances

```

call approve on wicp for marketplace to access x tokens
call makeOffer for x amount of tokens

```

### Accepting an offer -> `acceptOffer`

- to accept an offer, you must deposit nft at time of sale
- the offer amount is automatically withdrawn from the buyer to makretplace, and will attempt to send to the seller, if unsuccessful will fallback to balance that seller can manually withdraw
- after an offer is accepted, the offer is removed but the others remain (until denied or cancelled) and can still be accepted by the new owner

```

call approve on crowns canister for marketplace on token
call depositNFT on marketplace canister
call acceptOffer on marketplace canister

```

### Cancelling an offer -> `cancelOffer`

```

call cancelOffer on marketplace canister

```

## Withdraw methods

These are provided as a fallback way to withdraw a purchased nft or funds that failed to auto transfer

### Fungible (wicp) -> `withdrawFungible`

- When you withdraw a fungible token, it will withdraw the amount a user has that is not locked
- Users should only have a balance if a automatic withdraw failed, or they deposited more than they offered

### NFT (crowns) -> `withdrawNFT`

- you can only withdraw a token that is not listed for direct buy

```

```

