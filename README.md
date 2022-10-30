# CosmWasm <> Band integration

## Packages

Contain common data type that specific to BandChain oracle packet and some common input/output to request data to BandChain.

## Contracts

We provide sample contracts that either implement or consume these packages to both provide examples, and provide a basis for code you can extend for more custom contacts.

Price feed

- Price feed contract
  - Update new rate by request data to BandChain via IBC
  - Allow other contracts to query data from this contract
