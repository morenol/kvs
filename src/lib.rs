
use std::collections::HashMap;

/// Key-Value store structure.
pub struct KvStore{
   /// A HashMap from the std lib is used to store the key,value elements.
   m_hash: HashMap<String,String>,
}

impl KvStore{
   /// Get value of a given key in the KV store.
   ///
   ///```
   ///use kvs::KvStore;
   ///let store = KvStore::new();
   ///let missing = store.get("missing_key".to_owned());
   ///assert_eq!(missing, None);
   ///```
   pub fn get(&self, key: String) -> Option<String> {
      let value = self.m_hash.get(&key);
      match value {
            Some(s) => Some(s.clone()),
            _ => None
        }
   }

   /// Initialize an empty KvStore
   /// ```
   /// use kvs::KvStore;
   /// let store = KvStore::new();
   /// ```
   pub fn new() -> KvStore {
       let m_hash = HashMap::new();
       KvStore{
           m_hash
       }
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
   pub fn set(&mut self, key: String, value: String) {
      self.m_hash.insert(key, value);
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

   pub fn remove(&mut self, key: String) {
       self.m_hash.remove(&key);
   }

}

