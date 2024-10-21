use crate::domain;

pub type Id = domain::Id<Product>;
#[derive(Debug, Clone)]
pub struct Product {
    pub id: Id,
    pub source: Source,
    pub detail_url: url::Url,
    pub title: Option<String>,
    pub image_urls: Vec<url::Url>,
    pub retail_price: Option<String>,
    pub actual_price: Option<String>,
    pub retail_off: Option<String>,
    pub breadcrumb: Vec<String>,
    pub points: Option<String>,
}
impl Product {
    pub fn new(id: Id, source: Source, detail_url: url::Url, points: Option<String>) -> Self {
        Self {
            id,
            source,
            detail_url,
            title: None,
            image_urls: vec![],
            retail_price: None,
            actual_price: None,
            retail_off: None,
            breadcrumb: vec![],
            points,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, strum_macros::EnumString, strum_macros::Display)]
pub enum Source {
    Rakuten,
}
