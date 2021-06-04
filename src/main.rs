use std::str::FromStr;

#[tokio::main]
async fn main() {
    let rest = bitcoin_rest::new(bitcoin_rest::DEFAULT_ENDPOINT);
    println!("{}", rest.blockhashbyheight(0).await.unwrap());
    let txid = bitcoin::hash_types::Txid::from_str("c7d01280157c7d34ee9c9037e79076ecd42c1748fcfa2ead10cc9cbe19dcd195").unwrap();
    let tx = rest.tx(txid).await.unwrap();
    println!("{:?}", tx.txid());
    let blockid = bitcoin::hash_types::BlockHash::from_str("00000000a3bbe4fd1da16a29dbdaba01cc35d6fc74ee17f794cf3aab94f7aaa0").unwrap();
    let block = rest.block(blockid).await.unwrap();
    println!("{:?}", block);
    let headers = rest.headers(10, blockid).await.unwrap();
    println!("{:?}", headers);
    let chaininfo = rest.chaininfo().await.unwrap();
    println!("{:?}", chaininfo);
    let utxos = rest.getutxos(true, &vec![
        bitcoin::hash_types::Txid::from_str("e67a0550848b7932d7796aeea16ab0e48a5cfe81c4e8cca2c5b03e0416850114").unwrap(),
    ]).await.unwrap();
    println!("{:?}", utxos);
}
