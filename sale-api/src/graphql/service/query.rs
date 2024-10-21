use crate::graphql::errors;
use crate::graphql::service::types::product::Product;
use async_graphql::connection::Connection;
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
    ) -> Result<Connection<String, Product>, errors::Error> {
        let product_repo = ctx.data::<ddb::types::product::Repository>()?;
        let cursor = cursor.map(|v| Cursor::from(v));
        let products = product_repo
            .find_by_status(domain::product::Status::Active, cursor, Some(10))
            .await?;

        todo!()
    }

    async fn product(&self, ctx: &Context<'_>, id: ID) -> Result<Product, errors::Error> {
        todo!()
    }
}
