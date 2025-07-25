use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum TimeUnit {
    Second,
    Millisecond,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expiry {
    pub epoch: u128,
    pub unit: TimeUnit,
}

impl Expiry {
    pub fn to_millis(&self) -> u128 {
        match self.unit {
            TimeUnit::Second => self.epoch * 1000,
            TimeUnit::Millisecond => self.epoch,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now_in_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        now_in_millis > self.to_millis()
    }
}

pub struct Entry {
    pub key: String,
    pub value: String,
    pub expiry: Option<Expiry>,
}

#[async_trait::async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn set(&self, entry: Entry);
    async fn get(&self, key: &str) -> Option<String>;
    async fn entries(&self) -> Vec<Entry>;
}

#[derive(Default)]
pub struct InMemoryRepository {
    store: RwLock<HashMap<String, Entry>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Repository for InMemoryRepository {
    async fn set(&self, entry: Entry) {
        let mut store = self.store.write().await;
        store.insert(entry.key.clone(), entry);
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        let entry = store.get(key)?;

        if let Some(expiry) = &entry.expiry {
            if expiry.is_expired() {
                return None;
            }
        }

        Some(entry.value.clone())
    }

    async fn entries(&self) -> Vec<Entry> {
        let store = self.store.read().await;
        store
            .values()
            .map(|entry| Entry {
                key: entry.key.clone(),
                value: entry.value.clone(),
                expiry: entry.expiry.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
pub mod fixture {
    use super::Entry;
    use super::Repository;

    #[derive(Default)]
    pub struct DummyRepository;

    #[async_trait::async_trait]
    impl Repository for DummyRepository {
        async fn set(&self, _entry: Entry) {}
        async fn get(&self, _key: &str) -> Option<String> {
            None
        }
        async fn entries(&self) -> Vec<Entry> {
            vec![]
        }
    }
}
