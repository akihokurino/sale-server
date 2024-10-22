use crate::graphql::shared::types::DateTime;
use async_graphql::{Object, ID};
use derive_more::{From, Into};
use sale::domain;

#[derive(Debug, Clone, Into, From)]
pub struct Product(pub domain::product::Product);
#[Object]
impl Product {
    async fn id(&self) -> ID {
        ID(self.0.id.clone().into())
    }

    async fn source(&self) -> Source {
        self.0.source.clone().into()
    }

    async fn status(&self) -> Status {
        self.0.status.clone().into()
    }

    async fn detail_url(&self) -> String {
        self.0.detail_url.to_string()
    }

    async fn title(&self) -> Option<String> {
        self.0.title.clone()
    }

    async fn image_urls(&self) -> Vec<String> {
        self.0
            .image_urls
            .iter()
            .map(|url| url.to_string())
            .collect()
    }

    async fn retail_price(&self) -> Option<String> {
        self.0.retail_price.clone()
    }

    async fn actual_price(&self) -> Option<String> {
        self.0.actual_price.clone()
    }

    async fn retail_off(&self) -> Option<String> {
        self.0.retail_off.clone()
    }

    async fn breadcrumb(&self) -> Vec<String> {
        self.0.breadcrumb.clone()
    }

    async fn points(&self) -> Option<String> {
        self.0.points.clone()
    }

    async fn created_at(&self) -> DateTime {
        self.0.created_at.clone().into()
    }

    async fn updated_at(&self) -> DateTime {
        self.0.updated_at.clone().into()
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "sale::domain::product::Source")]
pub enum Source {
    Rakuten,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "sale::domain::product::Status")]
pub enum Status {
    Prepare,
    Active,
}
