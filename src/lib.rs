//! bitcoin_rest - A Bitcoin Core REST API wrapper library for Rust.
//! 
//! This library calls the [Bitcoin Core's REST API endpoint](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md) and
//! converts them to [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) objects.
//! 
//! For details, please see [Context](./struct.Context.html).

#[cfg(feature="softforks")]
use std::collections::HashMap;
use serde::Deserialize;
pub use bytes;
pub use reqwest;
pub use bitcoin;
use bitcoin::hash_types::{BlockHash, Txid};
use bitcoin::blockdata::block::{Block, BlockHeader};
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::Decodable;

pub const DEFAULT_ENDPOINT: &str = "http://localhost:8332/rest/";

#[derive(Debug, Clone, Deserialize)]
pub struct Softfork {
    #[serde(rename="type")]
    pub type_: String,
    pub active: bool,
    #[serde(default)]
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChainInfo {
    pub chain: String,
    pub blocks: u32,
    pub headers: u32,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub mediantime: u32,
    pub verificationprogress: f64,
    pub chainwork: String,
    pub pruned: bool,
    #[serde(default)]
    pub pruneheight: u32,
    #[cfg(feature="softforks")]
    pub softforks: HashMap<String, Softfork>,
    pub warnings: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(default)]
    pub req_sigs: u32,
    #[serde(rename="type")]
    pub type_: String,
    #[serde(default)]
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub height: u32,
    pub value: f64,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoData {
    pub chain_height: u32,
    pub chaintip_hash: String,
    pub bitmap: String,
    pub utxos: Vec<Utxo>,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    BitcoinEncodeError(bitcoin::consensus::encode::Error)
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<bitcoin::consensus::encode::Error> for Error {
    fn from(err: bitcoin::consensus::encode::Error) -> Self {
        Self::BitcoinEncodeError(err)
    }
}

/// `bitcoin_rest` context.
#[derive(Debug, Clone)]
pub struct Context {
    endpoint: String,
    client: reqwest::Client,
}

/// Create a new `bitcoin_rest` context.
///
/// The `endpoint` will be the string like "http://localhost:8332/rest/"
/// (Note: this string is available via `bitcoin_rest::DEFAULT_ENDPOINT`).
pub fn new(endpoint: &str) -> Context {
    Context {
        endpoint: endpoint.to_string(),
        client: reqwest::Client::new(),
    }
}

impl Context {
    /// Call the REST endpoint and parse it as a JSON.
    pub async fn call_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = String::new() + &self.endpoint + path + ".json";
        let result = self.client.get(url)
            .send().await?
            .json::<T>().await?;
        Ok(result)
    }
    /// Call the REST endpoint (binary).
    pub async fn call_bin(&self, path: &str) -> Result<bytes::Bytes, reqwest::Error> {
        let url = String::new() + &self.endpoint + path + ".bin";
        let result = self.client.get(url)
            .send().await?
            .bytes().await?;
        Ok(result)
    }
    /// Call the REST endpoint (hex).
    pub async fn call_hex(&self, path: &str) -> Result<String, reqwest::Error> {
        let url = String::new() + &self.endpoint + path + ".hex";
        let mut result = self.client.get(url)
            .send().await?
            .text().await?;
        // Trim last '\n'.
        result.pop();
        Ok(result)
    }
    /// Call the [/tx](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#transactions) endpoint.
    pub async fn tx(&self, txhash: &Txid) -> Result<Transaction, Error> {
        let path = String::from("tx/") + &txhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(Transaction::consensus_decode(result.as_ref())?)
    }
    /// Call the [/block](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blocks) endpoint.
    pub async fn block(&self, blockhash: &BlockHash) -> Result<Block, Error> {
        let path = String::from("block/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(Block::consensus_decode(result.as_ref())?)
    }
    /// Call the [/block/notxdetails](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blocks) endpoint.
    pub async fn block_notxdetails(&self, blockhash: &BlockHash) -> Result<BlockHeader, Error> {
        let path = String::from("block/notxdetails/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(BlockHeader::consensus_decode(result.as_ref())?)
    }
    /// Call the [/headers](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blockheaders) endpoint.
    pub async fn headers(&self, count: u32, blockhash: &BlockHash) -> Result<Vec<BlockHeader>, Error> {
        let path = String::from("headers/") + &count.to_string() + "/" + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        let mut ret = Vec::new();
        const BLOCK_HEADER_SIZE: usize = 80usize;
        let mut offset = 0;
        while offset < result.len() {
            ret.push(BlockHeader::consensus_decode(result[offset..(offset+BLOCK_HEADER_SIZE)].as_ref())?);
            offset += BLOCK_HEADER_SIZE;
        }
        Ok(ret)
    }
    /// Call the [/blockhashbyheight](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blockhash-by-height) endpoint.
    pub async fn blockhashbyheight(&self, height: u32) -> Result<BlockHash, Error> {
        let path = String::from("blockhashbyheight/") + &height.to_string();
        let result = self.call_bin(&path).await?;
        Ok(BlockHash::consensus_decode(result.as_ref())?)
    }
    /// Call the [/chaininfo](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#chaininfo) endpoint.
    pub async fn chaininfo(&self) -> Result<ChainInfo, Error> {
        let result: ChainInfo = self.call_json("chaininfo").await?;
        Ok(result)
    }
    /// Call the [/getutxos](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#query-utxo-set) endpoint.
    pub async fn getutxos(&self, checkmempool: bool, txids: &[Txid]) -> Result<UtxoData, Error> {
        let mut url = String::from("getutxos/");
        if checkmempool {
            url += "checkmempool/"
        }
        for (i, txid) in txids.iter().enumerate() {
            url += &(txid.to_string() + "-" + &i.to_string());
        }
        let result: UtxoData = self.call_json(&url).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    #[tokio::test]
    async fn reqwest_fail() {
        let rest = new("http://invalid-url/");
        assert!(rest.blockhashbyheight(0).await.is_err());
    }
    struct Fixture {
        rest_env_name: &'static str,
        genesis_block_hash: &'static str,
        txid_coinbase_block1: &'static str,
    }
    async fn decode_fail(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        assert!(rest.blockhashbyheight(0xFFFFFFFF).await.is_err());
    }
    async fn tx(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let tx = rest.tx(&Txid::from_str(f.txid_coinbase_block1).unwrap()).await.unwrap();
        assert_eq!(tx.txid().to_string(), f.txid_coinbase_block1);
    }
    async fn block(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(f.genesis_block_hash).unwrap();
        let block = rest.block(&blockid).await.unwrap();
        assert_eq!(block.block_hash().to_string(), f.genesis_block_hash);
    }
    async fn block_notxdetails(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(f.genesis_block_hash).unwrap();
        let blockheader = rest.block_notxdetails(&blockid).await.unwrap();
        assert_eq!(blockheader.block_hash().to_string(), f.genesis_block_hash);
    }
    async fn headers(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(f.genesis_block_hash).unwrap();
        let headers = rest.headers(1, &blockid).await.unwrap();
        assert_eq!(headers[0].block_hash().to_string(), f.genesis_block_hash);
    }
    async fn chaininfo(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let chaininfo = rest.chaininfo().await.unwrap();
        assert_eq!(chaininfo.chain, "main");
    }
    async fn blockhashbyheight(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        assert_eq!(rest.blockhashbyheight(0).await.unwrap().to_string(), f.genesis_block_hash);
    }
    async fn blockhashbyheight_hex(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockhash_hex = rest.call_hex("blockhashbyheight/0").await.unwrap();
        let blockhash = BlockHash::from_str(&blockhash_hex).unwrap();
        assert_eq!(blockhash.to_string(), f.genesis_block_hash);
    }
    async fn utxos(f: &Fixture) {
        let test_endpoint = std::env::var(f.rest_env_name).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let utxos = rest.getutxos(true, &vec![
            Txid::from_str(f.txid_coinbase_block1).unwrap(),
        ]).await.unwrap();
        assert!(utxos.chain_height > 0);
    }
    const BTC: Fixture = Fixture {
        rest_env_name: "BITCOIN_REST_ENDPOINT",
        genesis_block_hash: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        txid_coinbase_block1: "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
    };
    #[tokio::test] async fn btc_decode_fail          () { decode_fail          (&BTC).await; }
    #[tokio::test] async fn btc_tx                   () { tx                   (&BTC).await; }
    #[tokio::test] async fn btc_block                () { block                (&BTC).await; }
    #[tokio::test] async fn btc_block_notxdetails    () { block_notxdetails    (&BTC).await; }
    #[tokio::test] async fn btc_headers              () { headers              (&BTC).await; }
    #[tokio::test] async fn btc_chaininfo            () { chaininfo            (&BTC).await; }
    #[tokio::test] async fn btc_blockhashbyheight    () { blockhashbyheight    (&BTC).await; }
    #[tokio::test] async fn btc_blockhashbyheight_hex() { blockhashbyheight_hex(&BTC).await; }
    #[tokio::test] async fn btc_utxos                () { utxos                (&BTC).await; }
    const MONA: Fixture = Fixture {
        rest_env_name: "MONACOIN_REST_ENDPOINT",
        genesis_block_hash: "ff9f1c0116d19de7c9963845e129f9ed1bfc0b376eb54fd7afa42e0d418c8bb6",
        txid_coinbase_block1: "10067abeabcd96a1261bc542b16d686d083308304923d74cb8f3bab4209cc3b9",
    };
    #[tokio::test] async fn mona_decode_fail      () { decode_fail      (&MONA).await; }
    #[tokio::test] async fn mona_tx               () { tx               (&MONA).await; }
    #[tokio::test] async fn mona_block            () { block            (&MONA).await; }
    #[tokio::test] async fn mona_block_notxdetails() { block_notxdetails(&MONA).await; }
    #[tokio::test] async fn mona_headers          () { headers          (&MONA).await; }
    #[cfg(not(feature="softforks"))]
    #[tokio::test] async fn mona_chaininfo        () { chaininfo        (&MONA).await; }
    //#[tokio::test] async fn mona_blockhashbyheight() { blockhashbyheight(&MONA).await; }
    #[tokio::test] async fn mona_utxos            () { utxos            (&MONA).await; }
}
