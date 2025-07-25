use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::sync::RwLock;

pub struct Entry {
    pub key: String,
    pub value: String,
    pub expires_at: Option<u128>,
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

    fn now_in_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    fn is_expired(expires_at: Option<u128>) -> bool {
        match expires_at {
            Some(expires_at) => Self::now_in_millis() > expires_at,
            None => false,
        }
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

        if Self::is_expired(entry.expires_at) {
            None
        } else {
            Some(entry.value.clone())
        }
    }

    async fn entries(&self) -> Vec<Entry> {
        let store = self.store.read().await;
        store
            .values()
            .map(|entry| Entry {
                key: entry.key.clone(),
                value: entry.value.clone(),
                expires_at: entry.expires_at,
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
