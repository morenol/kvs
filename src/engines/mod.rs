use crate::command::Command;
use crate::error::Result;

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledStore;

pub trait KvsEngine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;

    fn exec_command(&self, command: Command) -> Result<Option<String>> {
        match command {
            Command::Rm(key) => {
                self.remove(key)?;
                Ok(None)
            }
            Command::Set(key, value) => {
                self.set(key, value)?;
                Ok(None)
            }
            Command::Get(key) => self.get(key),
        }
    }
}
