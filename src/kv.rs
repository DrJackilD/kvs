#![deny(missing_docs)]
//! This crate defines simple key-value storage
//! with basic create-read-delete operations
use crate::cache::InMemoryMapCache;
use crate::storage::FileStorage;
use failure::{err_msg, Error};
use serde::{Deserialize, Serialize};

/// Custom Result type to wrap all errors,
/// which possible during work with KvStore
pub type Result<T> = std::result::Result<T, Error>;

/// Represent different database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Log {
    Set(String, String),
    Remove(String),
}

/// Public trait, which should be implemented by all storages, which want to work as a KvStore.storage
pub trait Storage: Iterator<Item = Result<Log>> + Sized {
    /// Create new storage instance
    fn new(db_name: &str) -> Result<Self>;
    /// Write value to a internal storage
    fn write(&mut self, value: Log) -> Result<()>;
}

/// Public trait which should be implemented by all structs, which want to interact with KvStore as cache
pub trait Cache: Sized {
    /// Create new instance
    fn new() -> Result<Self>;
    /// Insert result to cache. Take ownership of `log`
    fn insert(&mut self, entry: Log) -> Result<()>;
    /// Get `Log` for given key. Return owned value.
    fn get(&self, key: &str) -> Result<Option<Log>>;
    /// Return mutable reference of Log for given key
    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Log>>;
}

/// Key-value database
pub struct KvStore<S, C> {
    storage: S,
    cache: C,
}

impl KvStore<FileStorage, InMemoryMapCache> {
    /// Return new instance of KvStore
    /// [`storage::FileStorage`] using as default storage.
    /// [`cache::InMemoryMapCache`] using as default cache.
    /// To set up different storage use `store.storage(T)` method
    pub fn new(db: &str) -> Result<Self> {
        Ok(Self {
            storage: FileStorage::new(db)?,
            cache: InMemoryMapCache::new()?,
        })
    }
}

impl<S: Storage, C: Cache> KvStore<S, C> {
    /// This method set storage of KvStore to provided in storage argument.
    /// By default, method `new` create `KvStore` with `FileStorage`
    #[allow(unused)]
    fn set_storage(mut self, storage: S) -> Self {
        self.storage = storage;
        self
    }

    /// This method set cache of KvStore to provided in cache argument
    /// By default, method `new` create KvStore with `inMemoryMapCache`
    #[allow(unused)]
    fn set_cache(mut self, cache: C) -> Self {
        self.cache = cache;
        self
    }

    fn _get_from_db(&mut self, key: &str) -> Result<Option<Log>> {
        // Re-create entry state from logs
        let log = self
            .storage
            .by_ref()
            .filter_map(|item| match item {
                Ok(log) => match &log {
                    Log::Set(k, _) | Log::Remove(k) if k == key => Some(log),
                    _ => None,
                },
                Err(_) => None,
            })
            .last();
        Ok(log)
    }

    /// Get cloned String value from storage stored with given `key`
    pub fn get(&mut self, key: &str) -> Result<String> {
        let log = match self.cache.get_mut(key)? {
            Some(l) => l.clone(),
            None => {
                let log = match self._get_from_db(&key)? {
                    Some(l) => l,
                    None => return Err(err_msg("Key not found")),
                };
                self.cache.insert(log.clone())?;
                log
            }
        };
        match log {
            Log::Set(_, v) => Ok(v.clone()),
            _ => Err(err_msg("Key not found")),
        }
    }

    /// Set `value` to storage behind given `key`
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let cmd = Log::Set(key.to_owned(), value.to_owned());
        self.cache.insert(cmd.clone())?;
        self.storage.write(cmd)
    }

    /// Remove key-value pair from storage
    pub fn remove(&mut self, key: &str) -> Result<()> {
        if self.get(key).is_err() {
            return Err(err_msg("Key not found"));
        }
        let cmd = Log::Remove(key.to_owned());
        self.cache.insert(cmd.clone())?;
        self.storage.write(cmd)?;

        Ok(())
    }
}
