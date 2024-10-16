use crate::domain::product::{Id, Product, Source};
use crate::errors::Kind::{Internal, NotFound};
use crate::infra::aws::dynamo::{AttrConv, HasTableName, MustPresent, TableRepository};
use crate::AppResult;
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;
use std::str::FromStr;

impl TryFrom<HashMap<String, AttributeValue>> for Product {
    type Error = String;
    fn try_from(mut v: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let id = v.remove("pk").must_present()?.must_s()?;
        let source = Source::from_str(&v.remove("source").must_present()?.must_s()?)
            .map_err(|_| "invalid source")?;
        let detail_url = v.remove("detailUrl").must_present()?.must_s()?;

        Ok(Self {
            id: Id::from(id),
            source,
            detail_url: url::Url::parse(&detail_url).map_err(|_| "invalid url")?,
            title: None,
            image_urls: vec![],
            retail_price: None,
            actual_price: None,
            retail_off: None,
            breadcrumb: vec![],
            points: None,
        })
    }
}
impl Into<HashMap<String, AttributeValue>> for Product {
    fn into(self) -> HashMap<String, AttributeValue> {
        [
            ("pk", Some(AttributeValue::from_s(self.id.as_str()))),
            ("sk", Some(AttributeValue::from_s("#"))),
            (
                "source",
                Some(AttributeValue::from_s(&self.source.to_string())),
            ),
            ("glk", Some(AttributeValue::from_s(Id::typename()))),
        ]
        .into_iter()
        .flat_map(|(k, v)| v.map(|v| (k.into(), v)))
        .collect()
    }
}

impl HasTableName for Product {
    fn table_name() -> String {
        "product".to_string()
    }
}

pub type Repository = TableRepository<Product>;
impl Repository {
    pub async fn get(&self, id: &Id) -> AppResult<Product> {
        let res = self
            .cli
            .get_item()
            .table_name(self.table_name())
            .set_key(Some(HashMap::from([
                ("pk".into(), AttributeValue::from_s(id.as_str())),
                ("sk".into(), AttributeValue::from_s("#")),
            ])))
            .send()
            .await?;

        res.item.map_or(Err(NotFound.into()), |v| {
            Ok(Product::try_from(v).map_err(Internal.withf())?)
        })
    }

    pub async fn put(&self, item: Product) -> AppResult<()> {
        self.cli
            .put_item()
            .table_name(self.table_name())
            .set_item(Some(item.clone().into()))
            .send()
            .await?;
        Ok(())
    }
}
