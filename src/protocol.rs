use crate::command::Command;
use crate::error::{Error, ErrorKind, Result};

use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;

pub struct StreamHandler {
    pub reader: BufReader<TcpStream>,
}

#[derive(Debug)]
pub enum Value {
    None,
    Command(Command),
    Error(String),
    String(String),
    Integer(i64),
}

const CRLF_BYTES: &[u8] = b"\r\n";

impl Value {
    pub fn encode(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        match self {
            Value::None => {
                res.push(b';');
            }
            Value::Command(cmd) => {
                res.push(b'!');
                res.extend_from_slice(cmd.to_string().as_bytes());
            }
            Value::Error(err) => {
                res.push(b'?');
                res.extend_from_slice(err.as_bytes())
            }
            Value::String(s) => {
                res.push(b'$');
                res.extend_from_slice(s.as_bytes())
            }
            Value::Integer(num) => {
                res.push(b'#');
                res.extend_from_slice(num.to_string().as_bytes());
            }
        }
        res.extend_from_slice(CRLF_BYTES);
        res
    }
}

impl StreamHandler {
    pub fn new(reader: BufReader<TcpStream>) -> Self {
        Self { reader }
    }

    pub fn decode(&mut self) -> Result<Value> {
        let mut res: Vec<u8> = Vec::new();
        self.reader
            .read_until(b'\n', &mut res)
            .map_err(|_err| Error::from(ErrorKind::InvalidData))?;

        let len = res.len();

        if len < 3 {
            return Err(Error::from(ErrorKind::DataTooShort(len)));
        }

        if !is_crlf(res[len - 2], res[len - 1]) {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        let bytes = res[1..len - 2].as_ref();
        match res[0] {
            // Value::String
            b'!' => parse_command(bytes).map(Value::Command),
            // Value::Error
            b'?' => parse_string(bytes).map(Value::Error),
            b'$' => parse_string(bytes).map(Value::String),
            // Value::Integer
            b'#' => parse_integer(bytes).map(Value::Integer),
            b';' => Ok(Value::None),
            prefix => Err(Error::from(ErrorKind::InvalidPrefix(prefix))),
        }
    }
}

#[inline]
fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
}

#[inline]
fn parse_string(bytes: &[u8]) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_err| Error::from(ErrorKind::InvalidData))
}

#[inline]
fn parse_integer(bytes: &[u8]) -> Result<i64> {
    let str_integer = parse_string(bytes)?;
    (str_integer.parse::<i64>()).map_err(|_err| Error::from(ErrorKind::InvalidData))
}

#[inline]
fn parse_command(bytes: &[u8]) -> Result<Command> {
    let str_command = parse_string(bytes)?;
    Command::from_str(&str_command)
}
