use crate::domain::product::{Id, Product, Source};
use crate::errors::Kind::{Internal, NotFound};
use crate::infra::aws::ddb::cursor::{entity_with_cursor_conv_from, Cursor, WithCursor};
use crate::infra::aws::ddb::index::{
    EvaluateKeyNamesProvider, SecondaryIndex, GENERAL_PRIMARY_INDEX,
};
use crate::infra::aws::ddb::{
    anchor_attr_value, condition_eq, query, EntityWithCursor, FromAttr, HasTableName, HasTypeName,
    TableRepository, ToAttr,
};
use crate::{AppResult, MustPresent};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;
use std::str::FromStr;

impl TryFrom<HashMap<String, AttributeValue>> for Product {
    type Error = String;
    fn try_from(mut v: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let source = Source::from_str(&v.remove("source").must_present()?.to_s()?)
            .map_err(|_| "invalid source")?;
        let detail_url = v.remove("detailUrl").must_present()?.to_s()?;

        Ok(Self {
            id: v.remove("pk").must_present()?.try_into()?,
            source,
            detail_url: url::Url::parse(&detail_url).map_err(|_| "invalid url")?,
            title: None,
            image_urls: vec![],
            retail_price: None,
            actual_price: None,
            retail_off: None,
            breadcrumb: vec![],
            points: Some(v.remove("points").must_present()?.to_s()?),
        })
    }
}
impl Into<HashMap<String, AttributeValue>> for Product {
    fn into(self) -> HashMap<String, AttributeValue> {
        [
            ("pk", Some(self.id.into())),
            ("sk", Some(anchor_attr_value())),
            ("source", Some(self.source.to_string().into_attr())),
            ("detailUrl", Some(self.detail_url.to_string().into_attr())),
            ("points", self.points.map(|v| v.into_attr())),
            ("glk", Some(Product::type_name().into_attr())),
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
impl HasTypeName for Product {
    fn type_name() -> String {
        "Product".to_string()
    }
}

const INDEX_SOURCE_CREATED_AT: SecondaryIndex = SecondaryIndex {
    name: "source-createdAt-index",
    hash_key: "source",
    range_key: Some("createdAt"),
    primary_index: &GENERAL_PRIMARY_INDEX,
};

pub type Repository = TableRepository<Product>;
impl Repository {
    pub async fn find_by_source(
        &self,
        source: Source,
        cursor: Option<Cursor>,
        limit: Option<i32>,
    ) -> AppResult<Vec<EntityWithCursor<Product>>> {
        let index = &INDEX_SOURCE_CREATED_AT;

        query(
            self.cli
                .query()
                .table_name(self.table_name())
                .index_name(index.name)
                .key_conditions(index.hash_key, condition_eq(source.to_string().into_attr()))
                .scan_index_forward(false)
                .with_cursor(cursor)
                .map_err(|v| Internal.with(v))?,
            limit,
            entity_with_cursor_conv_from(index.evaluate_key_names(), Product::try_from),
        )
        .await
        .map_err(|v| Internal.with(v))
    }

    pub async fn get(&self, id: &Id) -> AppResult<Product> {
        let res = self
            .cli
            .get_item()
            .table_name(self.table_name())
            .set_key(Some(HashMap::from([
                ("pk".into(), id.clone().into()),
                ("sk".into(), anchor_attr_value()),
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

    pub async fn delete(&self, id: &Id) -> AppResult<()> {
        self.cli
            .delete_item()
            .table_name(self.table_name())
            .set_key(Some(HashMap::from([
                ("pk".into(), id.clone().into()),
                ("sk".into(), anchor_attr_value()),
            ])))
            .send()
            .await?;
        Ok(())
    }
}
