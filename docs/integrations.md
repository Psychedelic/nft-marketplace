# 🤖 Integration

## Overview 

The following document provides a brief explanation on how the integration process came into play. It won't provide every single detail about it, but hopefully an overview to help onboarding.

Bare in mind that the integration looks into the development environment and as such will primarily discuss the local replica environment, as such, mainnet staging or any other network environments are out of scope.

Also, we'll use `Yarn` as our package manager to run our examples, but `npm run` can be used accordingly if that's your preference.

## 🤔 Requirements

- The pcrgrep cli (e.g. for macOS [pcrgreg](https://formulae.brew.sh/formula/pcre))
- A DFX identity set to use a [Plug exported PEM](/docs/dfx.md) identity

It's recommended to check how to import the [Plug exported PEM](/docs/dfx.md) identity, it'll save you a lot of time troubleshooting, as there are known issues or bugs when attempting to import via DFX.

## 🪚 Tools

A set of tools are provided to:
- Create mock identities that you might have control e.g. via Plug
- Generate mock data
- Create certain use-cases by triggering events e.g. make an offer

We'll start by learning how to generate mock tokens on a NFT Canister.

Assuming that the local replica and the services are running let's generate some mock data (if you don't know how, check the [README](/README.md), also noted that you need to understand the basics before taking the integration path exposed here).

## 🪙 Create non-fungible tokens mock

Let's imagine a scenario in the marketplace, where you (Plug exported PEM identity imported and set as your [DFX cli](https://smartcontracts.org/docs/developers-guide/cli-reference.html) identity), hold a given number of non-fungible tokens ([Crowns](https://github.com/Psychedelic/crowns)) and other users Alice and Bob hold others.

Our tool, will generate the tokens based in some parameters that you'll send and distribute it among these users (You, Alice and Bob).

This is done by executing the `mock:generate-tokens` which take as first argument the `NFT Canister Id` (a DIP-721, such as [Crowns](https://github.com/Psychedelic/crowns)) and a number of tokens to generate (which will be distributed to the user or actors in the story).

💡 As noted above, you should have all the required services running

```sh
yarn mock:generate-tokens <NFT Canister Id> <Number of tokens>
```

Here's what the process does for you:
- Prepare the identities via [DFX cli](https://smartcontracts.org/docs/developers-guide/cli-reference.html)
- Mint DIP721 tokens for the users (yourself, Alice and Bob)
- The "number of tokens" are evenly distributed to all users

The NFT Canister Id can be found in the logs of the `services:start` command you previously run; or get it dynamically.

Here's an example of getting the NFT Canister Id dynamically when generating 9 tokens.

```sh
yarn mock:generate-tokens $(cd ./nft-marketplace/DIP721 && dfx canister id nft) 9
```

💡 Evently distributed means that if you pass a number of tokens that equals 9, we'd have 3 tokens assigned to you, 3 tokens to Alice and 3 tokens to Bob, because these are distributed failry among the users in the system. This is done so that you have data to reason about, you can of course change this to whatever your needs are.

