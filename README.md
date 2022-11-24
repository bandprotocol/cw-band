# CW-Band

**Disclaimer** This library is in a very early stage of development. Please don't use it in production.

This repo contains `packages` and `contracts` folder to provide a library to build cosmwasm smart contract to interact with BandChain.

---

## Table Of Contents
- [Project structure](#Project-Structure)
- [BandChain - Oracle Scripts & Data Sources](./docs/bandchain_oracle_scripts_&_data_sources.md)
- [How to request data from BandChain via IBC](./docs/how_to_request_data_from_bandchain_via_ibc.md)
- [Packages - cw-band](./docs/packages_cw_band.md)
- [Contract example - Price feed](./docs/contracts_price_feed.md)
- [TODO] [Contract example - Consumer](./docs/contracts_consumer.md)
- [Setup relayer](./docs/setup_relayer.md)

---
## Project Structure

### Docs

All documents related to cw-band will be placed in this folder.
### Packages

This folder contains libraries for smart contracts to use. These libraries will help make the process of requesting data from BandChain easier.

  - `cw-band`: This library will provide all necessary data types that you will need to use when you want to request data from BandChain. You can look more into detail [here](./docs/packages_data_types.md)

### Contracts

We provide sample contracts that either implement or consume these packages to both provide examples, and provide a basis for code you can extend for more custom contacts.

  - `Price feed`: The example of contract that able to request data from BandChain by using cw-band package and allow other contracts to query the data. You can learn more detail in this [document](./docs/contracts_price_feed.md)
  - [TODO] `Consumer`: The example of contract that query price from price feed contract. You can learn more detail in this [document](./docs/contracts_consumer.md)

### Scripts

We provide useful script such as building contract command, contract deployment in this folder.

