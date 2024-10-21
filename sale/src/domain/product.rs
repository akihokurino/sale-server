use crate::domain;
use crate::domain::time::LocalDateTime;

pub type Id = domain::Id<Product>;
#[derive(Debug, Clone)]
pub struct Product {
    pub id: Id,
    pub source: Source,
    pub status: Status,
    pub detail_url: url::Url,
    pub title: Option<String>,
    pub image_urls: Vec<url::Url>,
    pub retail_price: Option<String>,
    pub actual_price: Option<String>,
    pub retail_off: Option<String>,
    pub breadcrumb: Vec<String>,
    pub points: Option<String>,
    pub created_at: LocalDateTime,
    pub updated_at: LocalDateTime,
}
impl Product {
    pub fn new(
        id: Id,
        source: Source,
        detail_url: url::Url,
        points: Option<String>,
        now: LocalDateTime,
    ) -> Self {
        Self {
            id,
            source,
            status: Status::Prepare,
            detail_url,
            title: None,
            image_urls: vec![],
            retail_price: None,
            actual_price: None,
            retail_off: None,
            breadcrumb: vec![],
            points,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(
        self,
        title: Option<String>,
        image_urls: Vec<url::Url>,
        retail_price: Option<String>,
        actual_price: Option<String>,
        retail_off: Option<String>,
        breadcrumb: Vec<String>,
        now: LocalDateTime,
    ) -> Self {
        Self {
            status: Status::Active,
            title,
            image_urls,
            retail_price,
            actual_price,
            retail_off,
            breadcrumb,
            updated_at: now,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, strum_macros::EnumString, strum_macros::Display)]
pub enum Source {
    Rakuten,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, strum_macros::EnumString, strum_macros::Display)]
pub enum Status {
    Prepare,
    Active,
}
