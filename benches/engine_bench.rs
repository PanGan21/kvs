use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::thread_pool::RayonThreadPool;
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::prelude::*;
use tempfile::TempDir;

fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.bench_function("kvs", |b| {
        b.to_async(FuturesExecutor).iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (
                    KvStore::<RayonThreadPool>::open(temp_dir.path(), 8).unwrap(),
                    temp_dir,
                )
            },
            |(store, _temp_dir)| async move {
                for i in 1..(1 << 12) {
                    store
                        .clone()
                        .set(format!("key{}", i), "value".to_string())
                        .await
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("sled", |b| {
        b.to_async(FuturesExecutor).iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (
                    SledKvsEngine::<RayonThreadPool>::new(sled::open(&temp_dir).unwrap(), 8)
                        .unwrap(),
                    temp_dir,
                )
            },
            |(db, _temp_dir)| async move {
                for i in 1..(1 << 12) {
                    db.clone()
                        .set(format!("key{}", i), "value".to_string())
                        .await
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_bench");
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("kvs_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8).unwrap();
            for key_i in 1..(1 << i) {
                tokio::spawn(
                    store
                        .clone()
                        .set(format!("key{}", key_i), "value".to_string()),
                );
            }
            let rng = SmallRng::from_seed([0; 32]);
            b.to_async(FuturesExecutor).iter(|| async {
                store
                    .clone()
                    .get(format!("key{}", rng.clone().gen_range(1..i.to_owned())))
                    .await
                    .unwrap();
            })
        });
    }
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("sled_{}", i), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let db =
                SledKvsEngine::<RayonThreadPool>::new(sled::open(&temp_dir).unwrap(), 8).unwrap();
            for key_i in 1..(1 << i) {
                tokio::spawn(db.clone().set(format!("key{}", key_i), "value".to_string()));
            }
            let rng = SmallRng::from_seed([0; 32]);
            b.to_async(FuturesExecutor).iter(|| async {
                db.clone()
                    .get(format!("key{}", rng.clone().gen_range(1..i.to_owned())))
                    .await
                    .unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);
