use async_graphql::dataloader::Loader;
use derive_more::{From, Into};
use sale::domain;
use sale::errors::AppError;
use sale::infra::aws::ddb;
use std::collections::HashMap;

#[derive(Debug, Clone, Into, From)]
pub struct ProductLoader(ddb::types::product::Repository);
impl Loader<domain::product::Id> for ProductLoader {
    type Value = domain::product::Product;
    type Error = AppError;

    async fn load(
        &self,
        keys: &[domain::product::Id],
    ) -> Result<HashMap<domain::product::Id, Self::Value>, Self::Error> {
        self.0.batch_get(keys).await
    }
}
