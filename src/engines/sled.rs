use super::KvsEngine;
use crate::error::{Error, ErrorKind, Result};
use sled::Db;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SledStore {
    store: Db,
}
impl SledStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        let st = sled::Config::new().path(path).flush_every_ms(None).open();
        match st {
            Ok(store) => Ok(SledStore { store }),
            Err(_err) => Err(Error::from(ErrorKind::SledError)),
        }
    }
}
impl KvsEngine for SledStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        let result = self.store.get(key);
        match result {
            Ok(Some(s)) => {
                let value = String::from_utf8(s.to_vec());
                match value {
                    Ok(vl) => Ok(Some(vl)),
                    _ => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Ok(None) => Ok(None),
            Err(_) => Err(Error::from(ErrorKind::SledError)),
        }
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        let result = self.store.insert(key, value.as_bytes());
        match result {
            Ok(_something) => {
                let res = self.store.flush();
                match res {
                    Ok(_something) => Ok(()),
                    Err(_err) => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Err(_) => Err(Error::from(ErrorKind::SledError)),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        let result = self.store.remove(key);
        match result {
            Ok(Some(_thing)) => {
                let res = self.store.flush();
                match res {
                    Ok(_something) => Ok(()),
                    Err(_err) => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Ok(None) => Err(Error::from(ErrorKind::KeyNotFound)),
            Err(_err) => Err(Error::from(ErrorKind::UnknownError)),
        }
    }
}
