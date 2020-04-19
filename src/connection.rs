use crate::error::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpStream, ToSocketAddrs};

use crate::protocol::{StreamHandler, Value};

pub struct Connection {
    stream: StreamHandler,
}

impl Connection {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let tcp =
            TcpStream::connect(addr).map_err(|_err| Error::from(ErrorKind::ConnectionError))?;
        let buffer = BufReader::new(tcp);
        Ok(Connection {
            stream: StreamHandler::new(buffer),
        })
    }

    pub fn from_stream(stream: TcpStream) -> Self {
        Connection {
            stream: StreamHandler::new(BufReader::new(stream)),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        let stream = self.stream.reader.get_mut() as &mut dyn Write;
        stream
            .write_all(buf)
            .map_err(|_err| Error::from(ErrorKind::ConnectionError))
    }

    pub fn read(&mut self) -> Result<Value> {
        self.stream.decode()
    }
}
