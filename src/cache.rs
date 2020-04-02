use crate::kv::{Cache, Log, Result};
use std::collections::HashMap;

pub struct InMemoryMapCache {
    cache: HashMap<String, Log>,
}

impl Cache for InMemoryMapCache {
    fn new() -> Result<Self> {
        Ok(Self {
            cache: HashMap::new(),
        })
    }

    fn insert(&mut self, log: Log) -> Result<()> {
        let key = match &log {
            Log::Set(k, _) => k.clone(),
            Log::Remove(k) => k.clone()
        };
        self.cache.insert(key, log);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Log>> {
        match self.cache.get(key) {
            Some(log) => Ok(Some(log.clone())),
            None => Ok(None),
        }
    }

    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Log>> {
        Ok(self.cache.get_mut(key))
    }
}
