#[macro_use]
extern crate criterion;

use criterion::BenchmarkId;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use kvs::client::create_client;
use kvs::command::Command;
use kvs::engines::{KvStore, KvsEngine, SledStore};
use kvs::server::KvsServer;
use kvs::thread_pool::{SharedQueueThreadPool, ThreadPool};
#[macro_use]
extern crate slog;
use kvs::protocol::Value;

use std::net::TcpListener;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use slog::{Discard, Logger};

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
                |(store, _temp_dir)| {
                    for i in 1..(1 << 8) {
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
            |(db, _temp_dir)| {
                for i in 1..(1 << 8) {
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
            let store = KvStore::open(temp_dir.path()).unwrap();
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
        let db = SledStore::open(temp_dir.path()).unwrap();
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

fn read_queued_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("w_sharedkvs");
    for threads in [1, 2, 4].iter() {
        group
            .bench_with_input(
                BenchmarkId::from_parameter(threads),
                threads,
                |b, &threads| {
                    let s = threads as u32;
                    let pool = SharedQueueThreadPool::new(s).unwrap();
                    let _log = Logger::root(Discard, o!());
                    let temp_dir = TempDir::new().unwrap();

                    let engine = KvStore::open(temp_dir.path()).unwrap();
                    let find_available_port = || {
                        (8000..9000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
                    };
                    let port = find_available_port().unwrap();
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                    let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                    std::thread::spawn(move || {
                        server.listen_and_serve().unwrap();
                    });
                    let mut client = create_client(addr).unwrap();
                    for key_i in 1..1000 {
                        client
                            .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                            .unwrap();
                    }
                    b.iter(|| {
                        for key_i in 1..1000 {
                            match client
                                .send_cmd(Command::Get(format!("key{}", key_i)))
                                .unwrap()
                            {
                                Value::String(result) => assert_eq!(result, "value"),
                                _ => assert_eq!(1, 0),
                            }
                        }
                    });
                },
            )
            .sample_size(50);
    }

    group.finish();
}

fn write_queued_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("r_sharedkvs");
    for threads in [1, 2, 4].iter() {
        group
            .bench_with_input(
                BenchmarkId::from_parameter(threads),
                threads,
                |b, &threads| {
                    let s = threads as u32;
                    let pool = SharedQueueThreadPool::new(s).unwrap();
                    let drain = Discard;
                    let _log = Logger::root(drain, o!());
                    let temp_dir = TempDir::new().unwrap();

                    let find_available_port = || {
                        (8000..9000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
                    };

                    let port = find_available_port().unwrap();
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                    let engine = KvStore::open(temp_dir.path()).unwrap();
                    let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                    std::thread::spawn(move || {
                        server.listen_and_serve().unwrap();
                    });

                    let mut client = create_client(addr).unwrap();

                    b.iter(|| {
                        for key_i in 1..1000 {
                            client
                                .send_cmd(Command::Set(
                                    format!("key{}", key_i),
                                    "value".to_string(),
                                ))
                                .unwrap();
                        }
                        for key_i in 1..1000 {
                            match client
                                .send_cmd(Command::Get(format!("key{}", key_i)))
                                .unwrap()
                            {
                                Value::String(result) => assert_eq!(result, "value"),
                                _ => assert_eq!(1, 0),
                            }
                        }
                    });
                },
            )
            .sample_size(50);
    }
    group.finish();
}
criterion_group!(
    benches,
    set_bench,
    get_bench,
    write_queued_kvstore,
    read_queued_kvstore
);
criterion_main!(benches);
