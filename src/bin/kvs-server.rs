#[macro_use]
extern crate clap;
use clap::{App, AppSettings};
use kvs::engines::{KvStore, KvsEngine, SledStore};
use kvs::error::{Error, ErrorKind, Result};
use kvs::server::KvsServer;
use std::env;
use std::net::ToSocketAddrs;
use std::path::Path;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
use crate::slog::{Drain, Logger};
use kvs::thread_pool::{SharedQueueThreadPool, ThreadPool};

fn main() -> Result<()> {
    let decorator = slog_term::PlainDecorator::new(std::io::stderr());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let _log = Logger::root(drain, o!());

    let yaml = load_yaml!("server-cli.yml");

    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    info!(_log, "Starting Kvs server version {}", crate_version!());
    info!(_log, "Listening on {}", addr);

    let engine_parameter = matches.value_of("engine");
    let current_engine = current_eng();
    let engine = match engine_parameter {
        Some("kvs") => match current_engine.as_ref().map(|s| &s[..]) {
            Some("kvs") | None => {
                info!(_log, "Using kvs engine");
                Some("kvs")
            }
            _ => return Err(Error::from(ErrorKind::UncompatibleEngine)),
        },
        Some("sled") => match current_engine.as_ref().map(|s| &s[..]) {
            Some("sled") | None => {
                info!(_log, "Using sled engine");
                Some("sled")
            }
            _ => return Err(Error::from(ErrorKind::UncompatibleEngine)),
        },
        Some(eng) => {
            error!(_log, "Wrong engine specified: {}", eng);
            return Err(Error::from(ErrorKind::InvalidEngine));
        }
        None => {
            info!(
                _log,
                "Engine not defined in the parameters, using current or default engine."
            );
            match current_engine.as_ref().map(|s| &s[..]) {
                Some("kvs") | None => Some("kvs"),
                _ => Some("sled"),
            }
        }
    };

    let pool = SharedQueueThreadPool::new(5)?;

    match engine {
        Some("kvs") => {
            let engine = KvStore::open(".")?;
            run_with(addr, engine, pool, _log)?;
        }
        Some("sled") => {
            let engine = SledStore::open(".")?;
            run_with(addr, engine, pool, _log)?;
        }
        _ => return Err(Error::from(ErrorKind::UnknownError)),
    }

    Ok(())
}

fn current_eng() -> Option<String> {
    if Path::new("./kvs.log").exists() {
        return Some("kvs".to_owned());
    } else if Path::new("./db").exists() {
        return Some("sled".to_owned());
    }
    None
}

pub fn run_with<E: KvsEngine, P: ThreadPool, A: ToSocketAddrs, L: Into<slog::Logger>>(
    addr: A,
    engine: E,
    pool: P,
    _log: L,
) -> Result<()> {
    let mut server = KvsServer::new(addr, engine, pool, _log)?;
    server.listen_and_serve()
}
