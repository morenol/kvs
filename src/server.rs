use crate::error::{Error, ErrorKind, Result};
use std::net::ToSocketAddrs;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use crate::connection::Connection;
use crate::engines::{Engine, KvStore, SledStore};
use crate::protocol::Value;
use crate::thread_pool::*;
use slog::Logger;

pub struct KvsServer<TP: ThreadPool> {
    listener: TcpListener,
    engine: Engine,
    pool: TP,
    logger: Logger,
}

impl<TP: ThreadPool> KvsServer<TP> {
    pub fn new<A: ToSocketAddrs, L: Into<slog::Logger>>(
        addr: A,
        eng: Option<&str>,
        pool: TP,
        logger: L,
    ) -> Result<Self> {
        let engine;
        let logger = logger.into();
        if let Some(store) = eng {
            engine = match store {
                "kvs" => {
                    if Path::new("./db").exists() {
                        return Err(Error::from(ErrorKind::UncompatibleEngine));
                    }
                    let storage = KvStore::open(".")?;
                    Engine::KvsEngine(storage)
                }
                "sled" => {
                    if Path::new("./kvs.log").exists() {
                        return Err(Error::from(ErrorKind::UncompatibleEngine));
                    }
                    let storage = SledStore::open(".")?;
                    Engine::SledKvsEngine(storage)
                }
                _ => {
                    error!(logger, "Invalid Engine.");
                    return Err(Error::from(ErrorKind::InvalidEngine));
                }
            }
        } else if !Path::new("./db").exists() {
            info!(logger, "Using kvs engine.");
            let storage = KvStore::open(".")?;
            engine = Engine::KvsEngine(storage)
        } else {
            info!(logger, "Using sled engine.");
            let storage = SledStore::open(".")?;
            engine = Engine::SledKvsEngine(storage)
        }

        let listener = TcpListener::bind(addr).unwrap();

        Ok(KvsServer {
            listener,
            engine,
            logger,
            pool,
        })
    }

    pub fn listen_and_serve(&mut self) -> Result<()> {
        for stream in self.listener.incoming() {
            let client = stream.map_err(|_err| Error::from(ErrorKind::ConnectionError))?;
            let engine = self.engine.clone();
            let logger = self.logger.clone();
            self.pool.spawn(move || {
                match handle_client(client, engine, &logger) {
                    Ok(_) => (),
                    Err(_err) => info!(logger, "There was a problem."),
                };
            });
        }
        Ok(())
    }
}

fn handle_client(stream: TcpStream, engine: Engine, logger: &Logger) -> Result<()> {
    let mut conn = Connection::from_stream(stream);
    debug!(logger, "Handling new client");
    while match conn.read() {
        Ok(value) => {
            if let Value::Command(command) = value {
                let result = engine.exec_command(command);
                match result {
                    Ok(None) => {
                        let val = Value::None;
                        debug!(logger, "Sending {:?}", val);
                        conn.write(&val.encode()).unwrap();
                    }
                    Ok(Some(s)) => {
                        let val = Value::String(s);
                        debug!(logger, "Sending {:?}", val);
                        conn.write(&val.encode()).unwrap();
                    }
                    Err(ref err) => match err.kind() {
                        ErrorKind::KeyNotFound => {
                            let val = Value::Error("KeyNotFound".to_owned());
                            debug!(logger, "Sending {:?}", val);
                            conn.write(&val.encode()).unwrap();
                        }
                        _ => {
                            debug!(logger, "Something went wrong {:?}", err);
                        }
                    },
                }
            }
            true
        }
        Err(_err) => false,
    } {}
    debug!(logger, "Connection with client finished");

    Ok(())
}
