# nft-marketplace

In-development backend implementation of DIP721 (and soon multi-standard) NFT marketplace template canister.

## Running the Service dependencies

The following assumes that the local replica network is available and running. Here's an example of how to do it and keeping in the foreground (recommend for monitoring).

```sh
dfx start --clear
```

💡 If you don't know how to start a local replica check the documentations in the [Dfinity docs](https://smartcontracts.org/docs/quickstart/local-quickstart.html).

Deploy the Cap service to the local replica by running:

```sh
yarn cap:start
```

For the cases where you'd like to reset the Cap Service, run the command:

```sh
yarn cap:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).