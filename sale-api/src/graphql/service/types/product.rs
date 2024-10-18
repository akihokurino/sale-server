use async_graphql::{Object, ID};
use sale::domain;

pub struct Product(pub domain::product::Product);
#[Object]
impl Product {
    async fn id(&self) -> ID {
        ID(self.0.id.clone().into())
    }
}
