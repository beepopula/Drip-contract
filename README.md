Popula Drip Contract
==================

An implementation for non-transferable-fungible-token(NTFT), and a customized minting method.
See https://github.com/beepopula/near-non-transferable-token for more details

Exploring The Code
==================

## Terminology

* `owner_id`: The owner of this contract.
* `metadata`: regular fungible token metadata.
* `token`: Implementation for NTFT.
* `white_list`: A list of outer reputation source contracts.  

## Function specification

### ft_collect
The only minting method for contracts who wants to rely on this contract's account book and any other derivative functions. It collects specific method on other contracts through cross-contract call and gather those values to its account book. The only thing need to do for those contracts being called is to prove that the signer and the collector contract is correct.

## Build

Run `RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release` to build the project.
