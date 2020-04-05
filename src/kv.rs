#![deny(missing_docs)]
//! This crate defines simple key-value storage
//! with basic create-read-delete operations
use crate::cache::InMemoryMapCache;
use crate::storage::FileStorage;
use failure::{err_msg, Error};
use serde::{Deserialize, Serialize};

const UNCOMPACTED_THREESHOLD: usize = 1024 * 1024;

/// Custom Result type to wrap all errors,
/// which possible during work with KvStore
pub type Result<T> = std::result::Result<T, Error>;

/// Represent different database operations
/// last argument in all entries is a length of log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Log {
    Set(String, String),
    Remove(String),
}

/// Public trait, which should be implemented by all storages, which want to work as a KvStore.storage
pub trait Storage: Iterator<Item = Result<(Log, usize)>> + Sized {
    /// Create new storage instance
    fn new(db_name: &str) -> Result<Self>;
    /// Write value to a internal storage. Return result with amount of bytes writed
    fn write(&mut self, value: &Log) -> Result<usize>;
    /// Override WAL file by values in Vec<&Log>
    fn override_storage(&mut self, values: Vec<&Log>) -> Result<()>;
}

/// Public trait which should be implemented by all structs, which want to interact with KvStore as cache
pub trait Cache: Sized {
    /// Create new instance
    fn new() -> Result<Self>;
    /// Insert result to cache. Take ownership of `log`. Second argument is a size of log entry
    fn insert(&mut self, log: Log, size: usize) -> Result<()>;
    /// Get `Log` for given key. Return owned value.
    fn get(&self, key: &str) -> Result<Option<Log>>;
    /// Return mutable reference of Log for given key
    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Log>>;
    /// Return all logs in cache
    fn get_all(&self) -> Vec<&Log>;
    /// Return amount of space, which can be saved by removing old log entries
    fn uncompacted_space(&self) -> usize;
}

/// Key-value database
pub struct KvStore {
    storage: FileStorage,
    cache: InMemoryMapCache,
}

impl KvStore {
    /// Return new instance of KvStore
    /// [`storage::FileStorage`] using as default storage.
    /// [`cache::InMemoryMapCache`] using as default cache.
    pub fn new(db: &str) -> Result<Self> {
        let mut instance = Self {
            storage: FileStorage::new(db)?,
            cache: InMemoryMapCache::new()?,
        };
        instance.cache_logs()?;
        Ok(instance)
    }

    /// Compress sotrage by write only actuall values from cache, omitting old records
    /// This process consist of three steps:
    /// 1. Open new storage
    /// 2. Write all actual records from cache to it
    /// 3. Remove old storage
    /// Implementation left on the storage device, imlemented `Storage` trait via `Storage.override` function
    fn compress_storage(&mut self) -> Result<()> {
        self.storage.override_storage(self.cache.get_all())?;
        Ok(())
    }

    /// Load all log entries to cache
    fn cache_logs(&mut self) -> Result<()> {
        for item in self.storage.by_ref() {
            match item {
                Ok((log, size)) => self.cache.insert(log, size)?,
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    fn _get_from_db(&mut self, key: &str) -> Result<Option<(Log, usize)>> {
        // Re-create entry state from logs
        let log = self
            .storage
            .by_ref()
            .filter_map(|item| match item {
                Ok(log) => match &log {
                    (Log::Set(k, _), _) | (Log::Remove(k), _) if k == key => Some(log),
                    _ => None,
                },
                Err(_) => None,
            })
            .last();
        Ok(log)
    }

    /// Get cloned String value from storage stored with given `key`
    pub fn get(&mut self, key: &str) -> Result<String> {
        match self.cache.get_mut(key)? {
            Some(Log::Set(_, value)) => Ok(value.clone()),
            Some(Log::Remove(_)) => Err(err_msg("Key not found")),
            None => {
                let value = match self._get_from_db(&key)? {
                    Some((log, size)) => match &log {
                        Log::Set(_, value) => {
                            let v = value.clone();
                            self.cache.insert(log, size)?;
                            Some(v)
                        }
                        _ => None,
                    },
                    _ => None,
                };
                match value {
                    Some(v) => Ok(v),
                    None => Err(err_msg("Key not found")),
                }
            }
        }
    }

    /// Set `value` to storage behind given `key`
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let log = Log::Set(key.to_owned(), value.to_owned());
        let size = self.storage.write(&log)?;
        self.cache.insert(log, size)?;
        if self.cache.uncompacted_space() >= UNCOMPACTED_THREESHOLD {
            self.compress_storage()?
        }
        Ok(())
    }

    /// Remove key-value pair from storage
    pub fn remove(&mut self, key: &str) -> Result<()> {
        if self.get(key).is_err() {
            return Err(err_msg("Key not found"));
        }
        let log = Log::Remove(key.to_owned());
        let size = self.storage.write(&log)?;
        self.cache.insert(log, size)?;
        Ok(())
    }
}
