use crate::kv::{Cache, Log, Result};
use std::collections::HashMap;

pub struct InMemoryMapCache {
    cache: HashMap<String, SizedLog>,
    uncompacted: usize,
}

struct SizedLog {
    log: Log,
    size: usize,
}

impl SizedLog {
    fn new(log: Log, size: usize) -> Self {
        Self { log, size }
    }
}

impl Cache for InMemoryMapCache {
    fn new() -> Result<Self> {
        Ok(Self {
            cache: HashMap::new(),
            uncompacted: 0,
        })
    }

    fn insert(&mut self, log: Log, size: usize) -> Result<()> {
        match &log {
            Log::Remove(k) => {
                self.uncompacted += size;
                if let Some(l) = self.cache.remove(k) {
                    self.uncompacted += l.size;
                }
            },
            Log::Set(k, _) => {
                let old = self.cache.insert(k.clone(), SizedLog::new(log, size));
                if let Some(item) = old {
                    self.uncompacted += item.size
                }
            }
        }
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Log>> {
        match self.cache.get(key) {
            Some(sized_log) => Ok(Some(sized_log.log.clone())),
            None => Ok(None),
        }
    }

    fn get_mut(&mut self, key: &str) -> Result<Option<&mut Log>> {
        match self.cache.get_mut(key) {
            Some(sized_log) => Ok(Some(&mut sized_log.log)),
            None => Ok(None)
        }
    }

    fn get_all(&self) -> Vec<&Log> {
        let logs = self.cache.iter().map(|(_, v)| &v.log).collect();
        logs
    }

    fn uncompacted_space(&self) -> usize {
        self.uncompacted
    }
}
