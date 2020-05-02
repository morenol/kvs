#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use kvs::engines::{KvStore, KvsEngine, SledStore};
use rand::prelude::*;
use std::iter;
use tempfile::TempDir;

fn set_bench(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs",
        |b, _| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    (KvStore::open(temp_dir.path()).unwrap(), temp_dir)
                },
                |(mut store, _temp_dir)| {
                    for i in 1..(1 << 10) {
                        store.set(format!("key{}", i), "value".to_string()).unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        },
        iter::once(()),
    )
    .with_function("sled", |b, _| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (SledStore::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut db, _temp_dir)| {
                for i in 1..(1 << 9) {
                    db.set(format!("key{}", i), "value".to_string()).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    })
    .sample_size(33);
    c.bench("set_bench", bench);
}

fn get_bench(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs",
        |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            for key_i in 1..(1 << i) {
                store
                    .set(format!("key{}", key_i), "value".to_string())
                    .unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 16]);
            b.iter(|| {
                store
                    .get(format!("key{}", rng.gen_range(1, 1 << i)))
                    .unwrap();
            })
        },
        vec![8, 12],
    )
    .with_function("sled", |b, i| {
        let temp_dir = TempDir::new().unwrap();
        let mut db = SledStore::open(temp_dir.path()).unwrap();
        for key_i in 1..(1 << i) {
            db.set(format!("key{}", key_i), "value".to_string())
                .unwrap();
        }
        let mut rng = SmallRng::from_seed([0; 16]);
        b.iter(|| {
            db.get(format!("key{}", rng.gen_range(1, 1 << i))).unwrap();
        })
    })
    .sample_size(33);

    c.bench("get_bench", bench);
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);
