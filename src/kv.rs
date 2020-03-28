#![deny(missing_docs)]
//! This crate defines simple key-value storage
//! with basic create-read-delete operations
use crate::storage::FileStorage;
use failure::{format_err, Error};
use serde::{Deserialize, Serialize};
use serde_json;

/// Custom Result type to wrap all errors,
/// which possible during work with KvStore
pub type Result<T> = std::result::Result<T, Error>;

/// Represent different database operations
#[derive(Debug, Serialize, Deserialize)]
enum Log {
    Set(String, String),
    Remove(String),
}

/// Public trait, which should be implemented by all storages, which want to work as a KvStore.storage
pub trait Storage: Iterator<Item = String> + Sized {
    /// Create new storage instance
    fn new(db_name: String) -> Result<Self>;
    /// Write value to a internal storage
    fn write(&mut self, value: String) -> Result<()>;
}

/// Represent key-value entry from storage
/// Creating by read log based storage and re-create entry's state
#[derive(Debug)]
pub struct Entry {
    pub key: String,
    pub value: Option<String>,
}

impl Entry {
    fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
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
pub struct KvStore<T> {
    storage: T,
}

impl KvStore<FileStorage> {
    /// Return new instance of KvStore
    /// [`storage::FileStorage`] using as default storage.
    /// To set up different storage use `store.storage(T)` method
    pub fn new(db: &str) -> Result<Self> {
        Ok(Self {
            storage: FileStorage::new(String::from(db))?,
        })
    }
}

impl<T> KvStore<T>
where
    T: Storage,
{
    /// Get cloned String value from storage stored with given `key`
    pub fn get(&mut self, key: String) -> Result<Entry> {
        let mut entry = Entry::new(key.clone(), None);
        let cmds: Vec<Log> = self
            .storage
            .by_ref()
            .filter_map(|ref line| match serde_json::from_str(line) {
                Ok(cmd) => match &cmd {
                    Log::Set(k, _) | Log::Remove(k) if &key == k => Some(cmd),
                    _ => None,
                },
                Err(_) => None,
            })
            .collect();
        for cmd in cmds {
            entry.run(&cmd);
        }
        Ok(entry)
    }

    /// Set `value` to storage behind given `key`
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Log::Set(key, value);
        let serialized = serde_json::to_string(&cmd)?;
        self.storage.write(format!("{}\n", serialized))
    }

    /// Remove key-value pair from storage
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.get(key.clone()) {
            Ok(ent) => {
                if ent.value.is_none() {
                    return Err(format_err!("Key not found"));
                }
            }
            Err(err) => return Err(err),
        };
        let cmd = Log::Remove(key);
        let serialized = serde_json::to_string(&cmd)?;
        self.storage.write(format!("{}\n", serialized))?;
        Ok(())
    }
}
