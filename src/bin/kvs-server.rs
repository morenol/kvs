#[macro_use]
extern crate clap;
use clap::{App, AppSettings};
use kvs::error::Result;
use kvs::server::KvsServer;
use std::env;

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

    let engine = matches.value_of("engine");
    match engine {
        Some("kvs") => info!(_log, "Using kvs engine"),
        Some("sled") => info!(_log, "Using sled engine"),
        Some(eng) => error!(_log, "Wrong engine specified: {}", eng),
        _ => info!(
            _log,
            "Engine not defined in the parameters, using current engine."
        ),
    }
    let pool = SharedQueueThreadPool::new(5)?;
    let mut server = KvsServer::new(addr, engine, pool, _log)?;
    server.listen_and_serve()?;

    Ok(())
}
