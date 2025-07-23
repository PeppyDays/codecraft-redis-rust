use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::sync::RwLock;

type Key = String;
type ValueWithExpiresAt = (String, Option<u128>);

#[async_trait::async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn set(&self, key: &str, value: &str, expires_after: Option<u128>);
    async fn get(&self, key: &str) -> Option<String>;
    async fn entries(&self) -> Vec<(Key, ValueWithExpiresAt)>;
}

#[derive(Default)]
pub struct InMemoryRepository {
    store: RwLock<HashMap<Key, ValueWithExpiresAt>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn now() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    fn is_expired(expires_at: Option<u128>) -> bool {
        match expires_at {
            Some(expires_at) => Self::now() > expires_at,
            None => false,
        }
    }
}

#[async_trait::async_trait]
impl Repository for InMemoryRepository {
    async fn set(&self, key: &str, value: &str, expires_after: Option<u128>) {
        let expires_at = expires_after.map(|a| Self::now() + a);
        let mut store = self.store.write().await;
        store.insert(key.to_string(), (value.to_string(), expires_at));
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        let (value, expires_at) = store.get(key)?;

        if Self::is_expired(*expires_at) {
            None
        } else {
            Some(value.clone())
        }
    }

    async fn entries(&self) -> Vec<(Key, ValueWithExpiresAt)> {
        let store = self.store.read().await;
        store
            .iter()
            .map(|(k, (v, e))| (k.clone(), (v.clone(), *e)))
            .collect()
    }
}

#[cfg(test)]
pub mod fixture {
    use super::Repository;

    #[derive(Default)]
    pub struct DummyRepository;

    #[async_trait::async_trait]
    impl Repository for DummyRepository {
        async fn set(&self, _key: &str, _value: &str, _expires_after: Option<u128>) {}
        async fn get(&self, _key: &str) -> Option<String> {
            None
        }
        async fn entries(&self) -> Vec<(String, (String, Option<u128>))> {
            vec![]
        }
    }
}
