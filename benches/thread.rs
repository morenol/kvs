
#[macro_use]
extern crate slog;
use slog::{Discard, Logger};


#[macro_use]
extern crate criterion;
use criterion::BenchmarkId;
use kvs::client::create_client;
use kvs::command::Command;
use kvs::protocol::Value;
use kvs::server::KvsServer;
use kvs::thread_pool::{RayonThreadPool, SharedQueueThreadPool, ThreadPool};

use kvs::engines::{KvStore, SledStore};
use std::net::TcpListener;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use criterion::Criterion;
use tempfile::TempDir;

fn read_queued_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("r_sharedkvs");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = SharedQueueThreadPool::new(s).unwrap();
                let _log = Logger::root(Discard, o!());
                let temp_dir = TempDir::new().unwrap();

                let engine = KvStore::open(temp_dir.path()).unwrap();
                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());
                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });
                let mut client = create_client(addr).unwrap();
                for key_i in 1..500 {
                    client
                        .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                        .unwrap();
                }
                b.iter(|| {
                    for key_i in 1..500 {
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
        );
    }

    group.finish();
}

fn write_queued_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("w_sharedkvs");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = SharedQueueThreadPool::new(s).unwrap();
                let drain = Discard;
                let _log = Logger::root(drain, o!());
                let temp_dir = TempDir::new().unwrap();

                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());

                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                let engine = KvStore::open(temp_dir.path()).unwrap();
                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });

                let mut client = create_client(addr).unwrap();

                b.iter(|| {
                    for key_i in 1..500 {
                        client
                            .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                            .unwrap();
                    }
                    for key_i in 1..500 {
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
        );
    }
    group.finish();
}

fn read_rayon_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("r_rayon_kvs");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = RayonThreadPool::new(s).unwrap();
                let _log = Logger::root(Discard, o!());
                let temp_dir = TempDir::new().unwrap();

                let engine = KvStore::open(temp_dir.path()).unwrap();
                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());
                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });
                let mut client = create_client(addr).unwrap();
                for key_i in 1..500 {
                    client
                        .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                        .unwrap();
                }
                b.iter(|| {
                    for key_i in 1..500 {
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
        );
    }

    group.finish();
}

fn write_rayon_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("w_rayonkvs");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = RayonThreadPool::new(s).unwrap();
                let drain = Discard;
                let _log = Logger::root(drain, o!());
                let temp_dir = TempDir::new().unwrap();

                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());

                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                let engine = KvStore::open(temp_dir.path()).unwrap();
                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });

                let mut client = create_client(addr).unwrap();

                b.iter(|| {
                    for key_i in 1..500 {
                        client
                            .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                            .unwrap();
                    }
                    for key_i in 1..500 {
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
        );
    }
    group.finish();
}

fn read_rayon_sled(c: &mut Criterion) {
    let mut group = c.benchmark_group("r_rayon_sled");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = RayonThreadPool::new(s).unwrap();
                let _log = Logger::root(Discard, o!());
                let temp_dir = TempDir::new().unwrap();

                let engine = SledStore::open(temp_dir.path()).unwrap();
                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());
                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });
                let mut client = create_client(addr).unwrap();
                for key_i in 1..500 {
                    client
                        .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                        .unwrap();
                }
                b.iter(|| {
                    for key_i in 1..500 {
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
        );
    }

    group.finish();
}

fn write_rayon_sled(c: &mut Criterion) {
    let mut group = c.benchmark_group("w_rayon_sle");
    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &threads| {
                let s = threads as u32;
                let pool = RayonThreadPool::new(s).unwrap();
                let drain = Discard;
                let _log = Logger::root(drain, o!());
                let temp_dir = TempDir::new().unwrap();

                let find_available_port =
                    || (8000..14000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok());

                let port = find_available_port().unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                let engine = SledStore::open(temp_dir.path()).unwrap();

                let mut server = KvsServer::new(addr, engine, pool, _log).unwrap();

                std::thread::spawn(move || {
                    server.listen_and_serve().unwrap();
                });

                let mut client = create_client(addr).unwrap();

                b.iter(|| {
                    for key_i in 1..500 {
                        client
                            .send_cmd(Command::Set(format!("key{}", key_i), "value".to_string()))
                            .unwrap();
                    }
                    for key_i in 1..500 {
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
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    write_queued_kvstore,
    read_queued_kvstore,
    write_rayon_kvstore,
    read_rayon_kvstore,
    write_rayon_sled,
    read_rayon_sled,
);
criterion_main!(benches);
