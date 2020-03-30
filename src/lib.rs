use std::collections::HashMap;
use std::path::PathBuf;
use std::result;
use std::fs::{File, OpenOptions};
use failure::Error;
use serde::{Deserialize, Serialize};
use std::io::{BufReader,BufRead, BufWriter, Seek, SeekFrom, Write};
#[macro_use] extern crate failure;

pub type Result<T> = result::Result<T, Error>;

/// Key-Value store structure.
pub struct KvStore {
    /// A HashMap from the std lib is used to store the key,value elements.
    m_hash: HashMap<String, String>,

    /// File used to record the set and rm operations.
    file: File
}

impl KvStore {


    fn init(& mut self) -> Result<()> {
        let reader = BufReader::new(&self.file);

    for line in reader.lines(){
        let command : Command = serde_json::from_str(&line?[..])?;
        match command {
            Command::Rm(key) => self.m_hash.remove(&key),
            Command::Set(key, value) => self.m_hash.insert(key, value)
        };
    }
    Ok(())
    }

    fn exists(& self, key: String) -> bool {
        match self.m_hash.get(&key) {
            Some(_s) => true,
            None => false
        }
    }

    /// Get value of a given key in the KV store.
    ///
    ///```
    ///use kvs::KvStore;
    ///let store = KvStore::new();
    ///let missing = store.get("missing_key".to_owned());
    ///assert_eq!(missing, None);
    ///```
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let value = self.m_hash.get(&key);
        match value {
            Some(s) => Ok(Some(s.clone())),
            _ => Ok(None),
        }
    }

    fn log(&mut self, command: &Command) -> Result<()> {
        let mut wr = BufWriter::new(&self.file);
        wr.seek(SeekFrom::End(0))?;
        serde_json::to_writer(wr, &command).unwrap();

        let mut wr = BufWriter::new(&self.file);
        wr.seek(SeekFrom::End(0))?;
        wr.write_all(b"\n")?;
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path: PathBuf = path.into();

        path.push("kvs.log");

        let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(path)?;

        let m_hash = HashMap::new();

        let mut storage = KvStore{
            m_hash,
            file,
        };

        storage.init()?;

        Ok(storage)
    }


    ///
    /// Set value of a key in in the KV store
    /// ```
    /// use kvs::KvStore;
    /// let mut store = KvStore::new();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// let value = store.get("key1".to_owned()).unwrap();
    /// assert_eq!(value, "value1");
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set(key.clone(), value.clone());
        self.log(&command)?;
        self.m_hash.insert(key, value);
        Ok(())
    }

    /// Remove key-value from the KV store
    /// ```
    /// use kvs::KvStore;
    /// let mut store = KvStore::new();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// let value = store.get("key1".to_owned()).unwrap();
    /// assert_eq!(value, "value1");
    /// store.remove("key1".to_owned());
    /// let missing = store.get("missing_key".to_owned());
    /// assert_eq!(missing, None);
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        let command = Command::Rm(key.clone());
        match self.exists(key.clone()){
            false => Err(format_err!("Key not found")),
            true => {
                self.log(&command)?;
                self.m_hash.remove(&key);
                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Rm(String),
    Set(String, String)
}
