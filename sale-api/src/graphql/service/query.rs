use crate::graphql::errors;
use crate::graphql::service::types::product::Product;
use async_graphql::{Context, MergedObject, Object, ID};
use sale::domain;

#[derive(MergedObject, Default)]
pub struct QueryRoot(DefaultQuery);

#[derive(Default)]
pub struct DefaultQuery;
#[Object]
impl DefaultQuery {
    async fn product(&self, ctx: &Context<'_>, id: ID) -> Result<Product, errors::Error> {
        Ok(Product(domain::product::Product {
            id: domain::product::Id::generate(),
            source: domain::product::Source::Rakuten,
            detail_url: url::Url::parse("https://example.com").unwrap(),
            title: None,
            image_urls: vec![],
            retail_price: None,
            actual_price: None,
            retail_off: None,
            breadcrumb: vec![],
            points: None,
        }))
    }
}
