use std::collections::HashMap;

use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn set(&self, key: &str, value: &str);
    async fn get(&self, key: &str) -> Option<String>;
}

#[derive(Default)]
pub struct InMemoryRepository {
    store: RwLock<HashMap<String, String>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Repository for InMemoryRepository {
    async fn set(&self, key: &str, value: &str) {
        let mut store = self.store.write().await;
        (*store).insert(key.to_string(), value.to_string());
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        (*store).get(key).cloned()
    }
}
