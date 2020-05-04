use crate::error::{Error, ErrorKind, Result};
use std::net::ToSocketAddrs;

use crate::command::Command;
use crate::connection::Connection;
use crate::protocol::Value;

pub fn create_client<A: ToSocketAddrs>(address: A) -> Result<KvsClient> {
    let client = KvsClient::new(address).map_err(|_err| Error::from(ErrorKind::ConnectionError))?;
    Ok(client)
}

pub struct KvsClient {
    conn: Connection,
}

impl KvsClient {
    fn new<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        let connection = Connection::new(addrs).unwrap();
        Ok(KvsClient { conn: connection })
    }

    pub fn send_cmd(&mut self, command: Command) -> Result<Value> {
        let value = Value::Command(command);
        self.send(&value.encode()).unwrap();
        self.read()
    }

    pub fn send(&mut self, value: &[u8]) -> Result<()> {
        self.conn.write(value)
    }

    pub fn read(&mut self) -> Result<Value> {
        self.conn.read()
    }
}
