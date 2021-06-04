use std::str::FromStr;
use std::collections::HashMap;
use serde::Deserialize;
use bitcoin::consensus::Decodable;

pub const DEFAULT_ENDPOINT: &str = "http://localhost:8332/rest/";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPubKey {
    asm: String,
    hex: String,
    #[serde(default)]
    req_sigs: u32,
    #[serde(rename="type")]
    type_: String,
    #[serde(default)]
    addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Softfork {
    #[serde(rename="type")]
    type_: String,
    active: bool,
    height: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChainInfo {
    chain: String,
    blocks: u32,
    headers: u32,
    bestblockhash: String,
    difficulty: f64,
    mediantime: u32,
    verificationprogress: f64,
    chainwork: String,
    pruned: bool,
    #[serde(default)]
    pruneheight: u32,
    softforks: HashMap<String, Softfork>,
    warnings: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    height: u32,
    value: f64,
    script_pub_key: ScriptPubKey,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoData {
    chain_height: u32,
    chaintip_hash: String,
    bitmap: String,
    utxos: Vec<Utxo>,
}

pub struct Context {
    endpoint: String,
}

impl Context {
    /// Call the REST endpoint.
    pub async fn call_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let url = String::new() + &self.endpoint + path + ".json";
        let result = reqwest::get(url)
            .await?
            .json::<T>()
            .await?;
        Ok(result)
    }
    pub async fn call_bin(&self, path: &str) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
        let url = String::new() + &self.endpoint + path + ".bin";
        let result = reqwest::get(url)
            .await?
            .bytes()
            .await?;
        Ok(result)
    }
    pub async fn call_hex(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = String::new() + &self.endpoint + path + ".hex";
        let mut result = reqwest::get(url)
            .await?
            .text()
            .await?;
        // Trim last '\n'.
        result.pop();
        Ok(result)
    }
    pub async fn tx(&self, txhash: bitcoin::hash_types::Txid)
        -> Result<bitcoin::blockdata::transaction::Transaction, Box<dyn std::error::Error>> {
        let path = String::from("tx/") + &txhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(bitcoin::blockdata::transaction::Transaction::consensus_decode(result.as_ref())?)
    }
    pub async fn block(&self, blockhash: bitcoin::hash_types::BlockHash) ->
        Result<bitcoin::blockdata::block::Block, Box<dyn std::error::Error>> {
        let path = String::from("block/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(bitcoin::blockdata::block::Block::consensus_decode(result.as_ref())?)
    }
    pub async fn block_notxdetails(&self, blockhash: bitcoin::hash_types::BlockHash) ->
        Result<bitcoin::blockdata::block::BlockHeader, Box<dyn std::error::Error>> {
        let path = String::from("block/notxdetails/") + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        Ok(bitcoin::blockdata::block::BlockHeader::consensus_decode(result.as_ref())?)
    }
    pub async fn headers(&self, count: u32, blockhash: bitcoin::hash_types::BlockHash) ->
        Result<Vec<bitcoin::blockdata::block::BlockHeader>, Box<dyn std::error::Error>> {
        let path = String::from("headers/") + &count.to_string() + "/" + &blockhash.to_string();
        let result = self.call_bin(&path).await?;
        let mut ret = Vec::new();
        for i in 0..count {
            let begin = (i as usize) * 80usize;
            let end = ((i + 1) as usize) * 80usize;
            ret.push(bitcoin::blockdata::block::BlockHeader::consensus_decode(result.slice(begin .. end).as_ref())?);
        }
        Ok(ret)
    }
    pub async fn blockhashbyheight(&self, height: u32) -> Result<bitcoin::hash_types::BlockHash, Box<dyn std::error::Error>> {
        let path = String::from("blockhashbyheight/") + &height.to_string();
        let result = self.call_hex(&path).await?;
        Ok(bitcoin::hash_types::BlockHash::from_str(&result)?)
    }
    pub async fn chaininfo(&self) -> Result<ChainInfo, Box<dyn std::error::Error>> {
        let result: ChainInfo = self.call_json("chaininfo").await?;
        Ok(result)
    }
    pub async fn getutxos(&self, checkmempool: bool, txids: &Vec<bitcoin::hash_types::Txid>) ->
        Result<UtxoData, Box<dyn std::error::Error>> {
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

pub fn new(endpoint: &str) -> Context {
    Context{ endpoint: endpoint.to_string() }
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
        let tx = rest.tx(bitcoin::hash_types::Txid::from_str(TXID_COINBASE_BLOCK1).unwrap()).await.unwrap();
        assert_eq!(tx.txid().to_string(), TXID_COINBASE_BLOCK1);
    }
    #[tokio::test]
    async fn block() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = bitcoin::hash_types::BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let block = rest.block(blockid).await.unwrap();
        assert_eq!(block.block_hash().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn block_notxdetails() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = bitcoin::hash_types::BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let blockheader = rest.block_notxdetails(blockid).await.unwrap();
        assert_eq!(blockheader.block_hash().to_string(), GENESIS_BLOCK_HASH);
    }
    #[tokio::test]
    async fn headers() {
        let test_endpoint = std::env::var(REST_ENV_NAME).unwrap_or(DEFAULT_ENDPOINT.to_string());
        let rest = new(&test_endpoint);
        let blockid = bitcoin::hash_types::BlockHash::from_str(GENESIS_BLOCK_HASH).unwrap();
        let headers = rest.headers(1, blockid).await.unwrap();
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
            bitcoin::hash_types::Txid::from_str(TXID_COINBASE_BLOCK1).unwrap(),
        ]).await.unwrap();
        assert!(utxos.chain_height > 0);
    }
}
