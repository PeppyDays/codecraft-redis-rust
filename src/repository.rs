use std::collections::HashMap;
use std::time::SystemTime;

use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn set(&self, key: &str, value: &str, expires_after: Option<u128>);
    async fn get(&self, key: &str) -> Option<String>;
}

#[derive(Default)]
pub struct InMemoryRepository {
    store: RwLock<HashMap<String, (String, Option<u128>)>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Repository for InMemoryRepository {
    async fn set(&self, key: &str, value: &str, expires_after: Option<u128>) {
        let mut expires_at = None;
        if let Some(exp) = expires_after {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            expires_at = Some(now.as_millis() + exp);
        }
        let mut store = self.store.write().await;
        (*store).insert(key.to_string(), (value.to_string(), expires_at));
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        let (x, y) = (*store).get(key)?;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        if y.is_some() && now > y.unwrap() {
            return None;
        }
        Some(x.to_string())
    }
}
