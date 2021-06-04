use std::str::FromStr;
use std::collections::HashMap;
use serde::Deserialize;
use bitcoin::consensus::Decodable;

pub const DEFAULT_ENDPOINT: &str = "http://localhost:8332/rest/";

#[derive(Debug, Deserialize)]
pub struct BlockHashByHeight {
    blockhash: String,
}

#[derive(Debug, Deserialize)]
pub struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vin {
    txid: String,
    vout: u32,
    script_sig: ScriptSig,
    sequence: u32,
    txinwitness: Vec<String>,
}

impl Vin {
    pub fn to_vin(&self) -> Result<bitcoin::blockdata::transaction::TxIn, Box<dyn std::error::Error>> {
        Ok(bitcoin::blockdata::transaction::TxIn{
            previous_output: bitcoin::blockdata::transaction::OutPoint{
                txid: bitcoin::hash_types::Txid::from_str(&self.txid).unwrap(),
                vout: self.vout,
            },
            script_sig: bitcoin::blockdata::script::Script::from_str(&self.script_sig.hex).unwrap(),
            sequence: self.sequence,
            witness: self.txinwitness.iter().map(|witness| {
                let mut buf: Vec<u8> = Vec::new();
                buf.resize(witness.len() / 2, 0);
                hex::decode_to_slice(witness, &mut buf).unwrap();
                buf
            }).collect(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPubKey {
    asm: String,
    hex: String,
    req_sigs: u32,
    #[serde(rename="type")]
    type_: String,
    addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    value: f64,
    script_pub_key: ScriptPubKey,
}

impl Vout {
    pub fn to_vout(&self) -> Result<bitcoin::blockdata::transaction::TxOut, Box<dyn std::error::Error>> {
        Ok(bitcoin::blockdata::transaction::TxOut{
            value: (1e8 * self.value) as u64,
            script_pubkey: bitcoin::blockdata::script::Script::from_str(&self.script_pub_key.hex).unwrap(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Tx {
    version: i32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
}

impl Tx {
    pub fn to_transaction(&self) -> Result<bitcoin::blockdata::transaction::Transaction, Box<dyn std::error::Error>> {
        let vins = self.vin.iter().map(|vin| vin.to_vin()).collect::<Result<Vec<_>, _>>()?;
        let vouts = self.vout.iter().map(|vout| vout.to_vout()).collect::<Result<Vec<_>, _>>()?;
        Ok(bitcoin::blockdata::transaction::Transaction{
            version: self.version,
            lock_time: self.locktime,
            input: vins,
            output: vouts,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockHeader {
    version: i32,
    previousblockhash: String,
    merkleroot: String,
    time: u32,
    bits: String,
    nonce: u32,
}

macro_rules! to_block_header {
    ($self: expr) => {
        bitcoin::blockdata::block::BlockHeader{
            version: $self.version,
            prev_blockhash: bitcoin::hash_types::BlockHash::from_str(&$self.previousblockhash)?,
            merkle_root: bitcoin::hash_types::TxMerkleNode::from_str(&$self.merkleroot)?,
            time: $self.time,
            bits: u32::from_str_radix(&$self.bits, 16)?,
            nonce: $self.nonce,
        }
    }
}

impl BlockHeader {
    pub fn to_block_header(&self) -> Result<bitcoin::blockdata::block::BlockHeader, Box<dyn std::error::Error>> {
        Ok(to_block_header!(self))
    }
}

#[derive(Debug, Deserialize)]
pub struct Block {
    version: i32,
    previousblockhash: String,
    merkleroot: String,
    time: u32,
    bits: String,
    nonce: u32,
    tx: Vec<Tx>,
}

impl Block {
    pub fn to_block_header(&self) -> Result<bitcoin::blockdata::block::BlockHeader, Box<dyn std::error::Error>> {
        Ok(to_block_header!(self))
    }
    pub fn to_block(&self) -> Result<bitcoin::blockdata::block::Block, Box<dyn std::error::Error>> {
        let blockheader = self.to_block_header()?;
        let transactions = self.tx.iter().map(|tx| tx.to_transaction()).collect::<Result<Vec<_>, _>>()?;
        Ok(bitcoin::blockdata::block::Block{
            header: blockheader,
            txdata: transactions,
        })
    }
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
