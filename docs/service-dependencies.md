# Service dependencies

The following is a brief breakdown of the utility methods to help interact with the required Services for the Marketplace Service Canister. These are created for your own convinience and while it offers a good start, it does not attempt to replace the knowledge you can get by checking the original Service documentation (e.g. if you'd like to learn more about Cap, then you're advised to check the original repository [here](https://github.com/Psychedelic/cap) and so on for the remaining Services).

## Cap

[CAP](https://github.com/Psychedelic/cap) is an open internet service providing transaction history & asset provenance for NFT’s & Tokens on the Internet Computer. If you are interested in finding more, follow the link [here](https://github.com/Psychedelic/cap).

Deploy the Cap service to the local replica by running:

```sh
yarn cap:start
```

For the cases where you'd like to reset the Cap Service, run the command:

```sh
yarn cap:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).

## Marketplace

Deploy the Marketplace Service Canister to the local replica by running:

```sh
yarn marketplace:deploy
```

For the cases where you'd like to reset the Marketplace Service, run the command:

```sh
yarn marketplace:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).

## Dab

[Dab](https://github.com/Psychedelic/dab) is an open internet service for NFT, Token, Canister, and Dapp registries. If you're interested, learn more about it [here](https://github.com/Psychedelic/dab).

Deploy the Dab service to the local replica by running:

```sh
yarn dab:start
```

For the cases where you'd like to reset the Dab Service, run the command:

```sh
yarn dab:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).


## DIP-721

The DIP721 is an Internet Computer Non-fungible Token Standard. If you'd like to learn more about it, follow the link [here](https://github.com/Psychedelic/dip721).

To deploy an NFT Canister run:

```sh
yarn dip721:deploy-nft
```

Set a controller by running:

```sh
yarn dip721:set-controllers <Principal ID>
```

For the cases where you'd like to reset the DIP-721 Canister, run the command:

```sh
yarn dip721:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).

# wICP

[Wrapped ICP](https://github.com/Psychedelic/wicp) (WICP) is a composable and interoperable wrapped version of ICP. You might also be interested in finding about the [DIP-20](https://github.com/Psychedelic/DIP20), a fungible token standard for the DFINITY Internet Computer. If you'd like to learn more about them, click on the individual links to access the repositories!

To deploy the wICP Canister run:

```sh
yarn wicp:deploy
```

Check a balance of by:

```sh
yarn wicp:balance-of
```

For the cases where you'd like to reset the DIP-721 Canister, run the command:

```sh
yarn wicp:reset
```

💡 The reset clears the `.dfx` directory and the Rust artifacts are kept in the target directory. If you wish to clear the Rust artifacts, you must do it manually (e.g. rm -rf ./cap/target).
