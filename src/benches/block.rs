use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

async fn fetch_block(rest: &bitcoin_rest::Context, height: u32) {
    let blockhash = rest.blockhashbyheight(height).await.unwrap();
    let _block = rest.block(&blockhash);
}

fn bench(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let rest = bitcoin_rest::new(bitcoin_rest::DEFAULT_ENDPOINT);
    c.bench_function("Fetch block at height 1", |b| b.iter(|| {
        rt.block_on(async {
            fetch_block(&rest, 1).await;
        });
    }));
    c.bench_function("Fetch block at height 500000", |b| b.iter(|| {
        rt.block_on(async {
            fetch_block(&rest, 500_000).await;
        });
    }));
}

criterion_group!(benches, bench);
criterion_main!(benches);
