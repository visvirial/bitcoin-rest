bitcoin-rest
============

__bitcoin-rest__ is a Rust library for Bitcoin Core's REST API interface.

All API calls work with [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin).

Usage
-----

The following example fetches the genesis block and finally the `block` variable set to `bitcoin::blockdata::block::Block`
with the genesis block.

```rs
use bitcoin::hash_types::BlockHash;

let rest = bitcoin_rest::new(bitcoin_rest::DEAFULT_ENDPOINT);  // or new("http://HOSTNAME:PORT/rest/");
let blockid = BlockHash::from_str("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f").unwrap();
let block = rest.block(blockid).await.unwrap();
// block.block_hash().to_string() == "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
```

For REST API details, please see the [Unauthenticated REST Interface](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md) article on the Bitcoin Core's GitHub page.

