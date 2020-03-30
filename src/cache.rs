use crate::kv::{Cache, Entry, Result};
use std::collections::HashMap;

pub struct InMemoryMapCache {
    cache: HashMap<String, Entry>,
}

impl Cache for InMemoryMapCache {
    fn new() -> Result<Self> {
        Ok(Self {
            cache: HashMap::new(),
        })
    }

    fn insert(&mut self, entry: Entry) -> Result<()> {
        let key = entry.key.clone();
        self.cache.insert(key, entry);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Entry>> {
        match self.cache.get(key) {
            Some(entry) => Ok(Some(entry.clone())),
            None => Ok(None),
        }
    }

    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Entry>> {
        Ok(self.cache.get_mut(key))
    }
}
