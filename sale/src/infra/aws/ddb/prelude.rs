use crate::AppResult;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait BatchGet<E, K> {
    async fn batch_get(&self, ids: &[K]) -> AppResult<HashMap<K, E>>;
}
