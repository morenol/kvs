pub mod error;
pub use error::Result;
pub mod command;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate slog;

mod engines;
pub use engines::KvStore;
pub use engines::KvsEngine;

pub mod client;
pub mod connection;
pub mod protocol;
pub mod server;
