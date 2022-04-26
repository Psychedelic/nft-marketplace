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
  withdrawFungible : (principal, FungibleStandard) -> (Result);

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

To create a listing that is available for direct buy, the nft's operator must be set to the marketplace canister

```

call approve on crowns canister for marketplace on token
call makeListing with direct_buy = true

```

### Direct Buy NFT -> `directBuy`

to direct buy a listing, you must deposit funds before the direct buy will go through.

```

call approve on wicp for marketplace to access x tokens
call directBuy on marketplace canister for nft

```

## Offer Flow:

### Listing NFT for offers -> `makeListing`

To create a listing that is only available for offers, you only have to be the current owner of a given nft, and marketplace must be the token's operator

```

call makeListing with direct_buy = false

```

### Making an offer -> `makeOffer`

- to make an offer, you must set a proper allowance for marketplace for a users funds. This can be a one time, really high amount, or with each offer.
- If the allowance is set with each offer, the amount allowed for marketplace should always be equal or more than the total amount offered for a user.

```

call approve on wicp for marketplace to access x tokens
call makeOffer for x amount of tokens

```

### Accepting an offer -> `acceptOffer`

- the offer amount is automatically withdrawn from the buyer to makretplace, and will attempt to send to the seller, if unsuccessful will fallback to balance that seller can manually withdraw
- after an offer is accepted, the offer is removed but the others remain (until denied or cancelled) and can still be accepted by the new owner

```

call approve on crowns canister for marketplace on token
call acceptOffer on marketplace canister

```

### Cancelling an offer -> `cancelOffer`

```

call cancelOffer on marketplace canister

```

### Transaction fallback -> `withdrawFungible`

This method is provided as a fallback way to withdraw funds that were auto deposited, and a transaction failed.
If marketplace holds any fungible balance, a banner should pop up in frontends to allow users to withdraw.

