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
#[derive(Debug, Serialize, Deserialize)]
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
    /// Insert result to cache. Take ownership of `entry`
    fn insert(&mut self, entry: Entry) -> Result<()>;
    /// Get `Entry` for given key. Return owned value.
    fn get(&self, key: &str) -> Result<Option<Entry>>;
    /// Return mutable reference of Entry for given key
    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Entry>>;
}

/// Represent key-value entry from storage
/// Creating by read log based storage and re-create entry's state
#[derive(Debug, Clone)]
pub struct Entry {
    /// Key of this entry
    pub key: String,
    /// Value of this entry. For convenience it can be None sometime.
    pub value: Option<String>,
}

impl Entry {
    fn new(key: &str, value: Option<&str>) -> Self {
        Self {
            key: key.to_owned(),
            value: match value {
                Some(v) => Some(v.to_owned()),
                None => None,
            },
        }
    }

    fn run(&mut self, cmd: &Log) {
        match cmd {
            Log::Set(k, v) => {
                if &self.key == k {
                    self.value = Some(v.clone());
                }
            }
            Log::Remove(k) => {
                if &self.key == k {
                    self.value = None;
                }
            }
        }
    }
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

    fn _get_from_db(&mut self, key: &str) -> Result<Entry> {
        // Re-create entry state from logs
        let mut entry = Entry::new(key, None);
        let cmds = self.storage.by_ref().filter_map(|item| match item {
            Ok(log) => match &log {
                Log::Set(k, _) | Log::Remove(k) if k == key => Some(log),
                _ => None,
            },
            _ => None,
        });
        for cmd in cmds {
            entry.run(&cmd);
        }
        Ok(entry)
    }

    /// Get cloned String value from storage stored with given `key`
    pub fn get(&mut self, key: &str) -> Result<Entry> {
        // We try to get entry from cache first and if only if it didn't store - re-create from logs
        let entry = match self.cache.get_mut(key)? {
            Some(entry) => entry.clone(),
            None => {
                let entry = self._get_from_db(&key)?;
                self.cache.insert(entry.clone())?;
                entry
            }
        };
        if entry.value.is_none() {
            Err(err_msg("Key not found"))
        } else {
            Ok(entry)
        }
    }

    /// Set `value` to storage behind given `key`
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let cmd = Log::Set(key.to_owned(), value.to_owned());
        let entry = match self.get(key) {
            Ok(mut ent) => {
                ent.run(&cmd);
                ent
            }
            Err(_) => Entry::new(key, Some(value)),
        };
        self.cache.insert(entry)?;
        self.storage.write(cmd)
    }

    /// Remove key-value pair from storage
    pub fn remove(&mut self, key: &str) -> Result<()> {
        let mut entry = self.get(key)?;
        let cmd = Log::Remove(key.to_owned());
        entry.run(&cmd);
        self.cache.insert(entry)?;
        self.storage.write(cmd)?;

        Ok(())
    }
}
