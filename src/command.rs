use crate::error::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Get(String),
    Rm(String),
    Set(String, String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Get(key) => write!(f, "GET {}", key),
            Command::Rm(key) => write!(f, "RM {}", key),
            Command::Set(key, value) => write!(f, "SET {} {}", key, value),
        }
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let words = s.split_whitespace().collect::<Vec<_>>();
        match words[0] {
            "GET" => Ok(Command::Get(words[1].to_string())),
            "SET" => Ok(Command::Set(words[1].to_string(), words[2].to_string())),
            "RM" => Ok(Command::Rm(words[1].to_string())),
            _ => Err(Error::from(ErrorKind::InvalidCommand)),
        }
    }
}
