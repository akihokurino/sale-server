use crate::graphql::connection::connection_from;
use crate::graphql::data_loader::ProductLoader;
use crate::graphql::errors;
use crate::graphql::errors::not_found_error;
use crate::graphql::service::types::product::Product;
use async_graphql::connection::Connection;
use async_graphql::dataloader::Loader;
use async_graphql::{Context, MergedObject, Object, ID};
use sale::domain;
use sale::infra::aws::ddb;
use sale::infra::aws::ddb::cursor::Cursor;

#[derive(MergedObject, Default)]
pub struct QueryRoot(DefaultQuery);

#[derive(Default)]
pub struct DefaultQuery;
#[Object]
impl DefaultQuery {
    async fn products(
        &self,
        ctx: &Context<'_>,
        cursor: Option<String>,
        limit: Option<i32>,
    ) -> Result<Connection<String, Product>, errors::Error> {
        let product_repo = ctx.data::<ddb::types::product::Repository>()?;
        let cursor = cursor.map(|v| Cursor::from(v));
        let products = product_repo
            .find_by_status(domain::product::Status::Active, cursor, limit)
            .await?;
        Ok(connection_from(products, |v| Product::from(v)))
    }

    async fn product(&self, ctx: &Context<'_>, id: ID) -> Result<Product, errors::Error> {
        let product_loader = ctx.data::<ProductLoader>()?;
        let id = domain::product::Id::new(id.0);
        let res = product_loader.0.load(&[id.clone()]).await?;
        res.get(&id)
            .map(|v| Product::from(v.clone()))
            .ok_or(not_found_error())
    }
}
