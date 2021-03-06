![](https://storageapi.fleek.co/fleek-team-bucket/jelly-nft.png)

# 🍭 Jelly - An NFT Marketplace & Toolkit

[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-blue.svg)](https://conventionalcommits.org) [![Healthcheck](https://github.com/Psychedelic/nft-marketplace/actions/workflows/pr-healthcheck-runner.yml/badge.svg)](https://github.com/Psychedelic/nft-marketplace/actions/workflows/pr-healthcheck-runner.yml)

The NFT Marketplace is the backend Service for our DIP-721 implementation, and soon a multi-standard NFT Marketplace.

> 🚧 IMPORTANT 🚧 - Jelly is currently in it's first version of development as we move towards a full marketplace protocol. This repo should be treated and such and is subject to change. 

## 📒 Table of Contents

- [Service Flow Docs](/docs/service.md)
- [Requirements](#-requirements)
- [Getting started](#-getting-started)
- [Integrations](/docs/integrations.md)
  - [Tools](/docs/integrations.md#-tools)
  - [Create non-fungible tokens mock](/docs/integrations.md#-create-non-fungible-tokens-mock)
- [Contribution guideline](#-contributing)

### ⚙️ Requirements

TLDR; We're providing implementation examples and related test or use-cases for your convinence, if you are just interested in the specifications find it [here](spec.md).

The requirements listed here are for running the [DIP-721](spec.md) example implementations that are available in this repository. If you are just interested in the specification for [DIP-721](spec.md) follow the link [here](spec.md).

- Nodejs
- Yarn or NPM
- The [DFX SDK](https://smartcontracts.org/) to run the CLI

💡 During the guide we'll be using `yarn`, but this can be easily replaced by `npm`, if that's your preference.

### 🤔 Getting started

We'll use Nodejs `package.json` to provide and describe convenient methods to bootstrap, build or reset the state of the provided test cases.

Jelly interacts with [Cap](https://github.com/Psychedelic/cap), [Dab](https://github.com/Psychedelic/dab), [Crowns](https://github.com/Psychedelic/crowns) (a [DIP-721](https://github.com/Psychedelic/DIP721) token) and [wICP](https://github.com/Psychedelic/wicp) (a [DIP-20](https://github.com/Psychedelic/DIP20) token). If you haven't learn about these, learn more about them by clicking in the available links!

Jelly runs against these Service canisters, on mainnet and also within the local replica network when developing; As such these should be available in the network. For example, for local replica network, if you're already running the Service separatily on your own, feel free to skip the steps to initialise all the dependencies mentioned below. Otherwise, you have to pull and initialise the Git repositories far all the required Services as follows:

```sh
yarn services:init
```

You only need to do it once, for example, after you cloned the `Marketplace Services` repository.

> Note: Make sure you have the [DFX SDK](https://smartcontracts.org/) installed to run the DFX cli, otherwise visit the [Dfinity](https://dfinity.org/) for instructions

Launch the local replica in the foreground (you're advised to do it, to monitor the service, otherwise feel free to add the --background flag). You can open a new shell session afterwards while monitoring the local replica network.

```sh
dfx start --clean
```

## Running the Service dependencies

The following assumes that the local replica network is available and running. It's recommended to run the reset command everytime you need to initialse it.

```sh
yarn reset
```

💡 If you don't know how to start a local replica check the [getting started](#getting-started), or the documentations in the [Dfinity docs](https://smartcontracts.org/docs/quickstart/local-quickstart.html) for more details.

😅 TLDR; Run all the required services by executing the `services:start` command.

```sh
yarn services:start
```

Bare in mind that you'll have to deploy the DIP-721, set allowances, etc on your own, depending on your use-case.

⚠️ Regarding [Cap handshake](https://github.com/Psychedelic/cap/blob/main/docs/Rust-SDK.md#handshake), the root bucket is only created after the first insert! Until an event is inserted, there'll be no root bucket id; If you fail to consider this, it might cause confusion when you try to `get_user_root_buckets`, etc. As the root bucket id will not be available or provided!

The 🚑 [Healtcheck](./.scripts/healthcheck.sh) provides a description of how this can look like for the use-case where a DIP-721 is listed for sale, a user gets a sale offer and accepts it.

```sh
yarn marketplace:healthcheck
```

💡 If you'd like to know which services and how they are initialised then check the [Service dependencies](docs/service-dependencies.md) document, as this might be useful for troubleshooting.

## 🙏 Contributing

Create branches from the `main` branch and name it in accordance to **conventional commits** [here](https://www.conventionalcommits.org/en/v1.0.0/), or follow the examples bellow:

```txt
test: 💍 Adding missing tests
feat: 🎸 A new feature
fix: 🐛 A bug fix
chore: 🤖 Build process or auxiliary tool changes
docs: ✏️ Documentation only changes
refactor: 💡 A code change that neither fixes a bug or adds a feature
style: 💄 Markup, white-space, formatting, missing semi-colons...
```

Find more about contributing [here](docs/contributing.md), please!

