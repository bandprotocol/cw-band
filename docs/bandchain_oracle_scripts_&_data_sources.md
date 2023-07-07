# BandChain - Oracle scripts & Data sources

Band Protocol is a cross-chain data oracle aggregating and connecting real-world data and APIs to smart contracts.

The protocol is built on top of BandChain, a Cosmos-SDK-based blockchain designed to be compatible with most smart contract and blockchain development frameworks.

When you want to request data from BandChain. You will have to create two important parts on BandChain.
1. `Data sources`: It's an executable script that validators will help to run and report to result back on BandChain. You can look into more detail [here](https://docs.bandchain.org/develop/custom-scripts/data-source/introduction)

2. `Oracle scripts`: It's the script that will request data from validators and aggregate the results of data sources into the final answer. You can look into more detail [here](https://docs.bandchain.org/develop/custom-scripts/oracle-script/introduction)
