use async_graphql::dataloader::Loader;
use sale::domain;
use sale::errors::AppError;
use sale::infra::aws::ddb::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataLoader<E, K>(pub Arc<dyn BatchGet<E, K> + Send + Sync>);
impl<E, K> Loader<K> for DataLoader<E, K>
where
    E: Send + Sync + Clone + 'static,
    K: Hash + Eq + Send + Sync + Clone + 'static,
{
    type Value = E;
    type Error = AppError;

    async fn load(&self, keys: &[K]) -> Result<HashMap<K, Self::Value>, Self::Error> {
        self.0.batch_get(keys).await
    }
}

pub type ProductLoader = DataLoader<domain::product::Product, domain::product::Id>;
