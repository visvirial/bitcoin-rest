//! bitcoin_rest - A Bitcoin Core REST API wrapper library for Rust.
//! 
//! This library calls the [Bitcoin Core's REST API endpoint](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md) and
//! converts them to [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) objects.
//! For details, please see [Context](./struct.Context.html).

use std::str::FromStr;
use std::collections::HashMap;
use serde::Deserialize;
use bitcoin::hash_types::{BlockHash, Txid};
use bitcoin::blockdata::block::{Block, BlockHeader};
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::Decodable;

pub const DEFAULT_ENDPOINT: &str = "http://localhost:8332/rest/";

#[derive(Debug, Deserialize)]
pub struct Softfork {
    #[serde(rename="type")]
    pub type_: String,
    pub active: bool,
    #[serde(default)]
    pub height: u32,
}

#[derive(Debug, Deserialize)]
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
    pub softforks: HashMap<String, Softfork>,
    pub warnings: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub height: u32,
    pub value: f64,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoData {
    pub chain_height: u32,
    pub chaintip_hash: String,
    pub bitmap: String,
    pub utxos: Vec<Utxo>,
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
    pub async fn tx(&self, txhash: &Txid) -> Result<Transaction, reqwest::Error> {
        let path = String::from("tx/") + &txhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(Transaction::consensus_decode(result.as_ref()).unwrap())
    }
    /// Call the [/block](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blocks) endpoint.
    pub async fn block(&self, blockhash: &BlockHash) -> Result<Block, reqwest::Error> {
        let path = String::from("block/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(Block::consensus_decode(result.as_ref()).unwrap())
    }
    /// Call the [/block/notxdetails](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blocks) endpoint.
    pub async fn block_notxdetails(&self, blockhash: &BlockHash) -> Result<BlockHeader, reqwest::Error> {
        let path = String::from("block/notxdetails/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(BlockHeader::consensus_decode(result.as_ref()).unwrap())
    }
    /// Call the [/headers](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blockheaders) endpoint.
    pub async fn headers(&self, count: u32, blockhash: &BlockHash) -> Result<Vec<BlockHeader>, reqwest::Error> {
        let path = String::from("headers/") + &count.to_string() + "/" + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        let mut ret = Vec::new();
        for i in 0..count {
            let begin = (i as usize) * 80usize;
            let end = ((i + 1) as usize) * 80usize;
            ret.push(BlockHeader::consensus_decode(result.slice(begin .. end).as_ref()).unwrap());
        }
        Ok(ret)
    }
    /// Call the [/blockhashbyheight](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#blockhash-by-height) endpoint.
    pub async fn blockhashbyheight(&self, height: u32) -> Result<BlockHash, reqwest::Error> {
        let path = String::from("blockhashbyheight/") + &height.to_string();
        let result = self.call_hex(&path).await?;
        Ok(BlockHash::from_str(&result).unwrap())
    }
    /// Call the [/chaininfo](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#chaininfo) endpoint.
    pub async fn chaininfo(&self) -> Result<ChainInfo, reqwest::Error> {
        let result: ChainInfo = self.call_json("chaininfo").await?;
        Ok(result)
    }
    /// Call the [/getutxos](https://github.com/bitcoin/bitcoin/blob/master/doc/REST-interface.md#query-utxo-set) endpoint.
    pub async fn getutxos(&self, checkmempool: bool, txids: &Vec<Txid>) -> Result<UtxoData, reqwest::Error> {
        let mut url = String::from("getutxos/");
        if checkmempool {
            url += "checkmempool/"
        }
        for i in 0..txids.len() {
            url += &(txids[i].to_string() + "-" + &i.to_string());
        }
        let result: UtxoData = self.call_json(&url).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const REST_ENV_NAME: &str = "BITCOIN_REST_ENDPOINT";
    const GENESIS_BLOCK_HASH: &str = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
    const TXID_COINBASE_BLOCK1: &str = "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098";
    #[tokio::test]
    async fn tx() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let tx = rest.tx(&Txid::from_str(TXID_COINBASE_BLOCK1).unwrap()).await.unwrap();
        assert_eq!(tx.txid().to_string(), TXID_COINBASE_BLOCK1);
    }
    #[tokio::test]
    async fn block() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let block = rest.block(&blockid).await.unwrap();
        assert_eq!(block.block_hash().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn block_notxdetails() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let blockheader = rest.block_notxdetails(&blockid).await.unwrap();
        assert_eq!(blockheader.block_hash().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn headers() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let headers = rest.headers(1, &blockid).await.unwrap();
        assert_eq!(headers[0].block_hash().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn chaininfo() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let chaininfo = rest.chaininfo().await.unwrap();
        assert_eq!(chaininfo.chain, "main");
    }
    #[tokio::test]
    async fn blockhashbyheight() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        assert_eq!(rest.blockhashbyheight(0).await.unwrap().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn utxos() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let utxos = rest.getutxos(true, &vec![
            Txid::from_str(TXID_COINBASE_BLOCK1).unwrap(),
        ]).await.unwrap();
        assert!(utxos.chain_height > 0);
    }
}
