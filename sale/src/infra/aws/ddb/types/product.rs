use crate::domain::product::{Id, Product, Source, Status};
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
            .map_err(|_| "invalid enum")?;
        let status = Status::from_str(&v.remove("status").must_present()?.to_s()?)
            .map_err(|_| "invalid enum")?;
        let detail_url = v.remove("detailUrl").must_present()?.to_s()?;
        let image_urls = v
            .remove("imageUrls")
            .map(|v| v.to_s_list())
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            id: v.remove("pk").must_present()?.try_into()?,
            source,
            status,
            detail_url: url::Url::parse(&detail_url).map_err(|_| "invalid url".to_string())?,
            title: v.remove("title").map(|v| v.to_s()).transpose()?,
            image_urls: image_urls
                .iter()
                .map(|v| url::Url::parse(v).map_err(|_| "invalid url".to_string()))
                .collect::<Result<Vec<url::Url>, String>>()?,
            retail_price: v.remove("retailPrice").map(|v| v.to_s()).transpose()?,
            actual_price: v.remove("actualPrice").map(|v| v.to_s()).transpose()?,
            retail_off: v.remove("retailOff").map(|v| v.to_s()).transpose()?,
            breadcrumb: v
                .remove("breadcrumb")
                .map(|v| v.to_s_list())
                .transpose()?
                .unwrap_or_default(),
            points: Some(v.remove("points").must_present()?.to_s()?),
            created_at: v.remove("createdAt").must_present()?.to_date_time()?,
            updated_at: v.remove("updatedAt").must_present()?.to_date_time()?,
        })
    }
}
impl Into<HashMap<String, AttributeValue>> for Product {
    fn into(self) -> HashMap<String, AttributeValue> {
        [
            ("pk", Some(self.id.into())),
            ("sk", Some(anchor_attr_value())),
            ("source", Some(self.source.to_string().into_attr())),
            ("status", Some(self.status.to_string().into_attr())),
            ("detailUrl", Some(self.detail_url.to_string().into_attr())),
            ("title", self.title.map(|v| v.into_attr())),
            (
                "imageUrls",
                Some(
                    self.image_urls
                        .into_iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .into_attr(),
                ),
            ),
            ("retailPrice", self.retail_price.map(|v| v.into_attr())),
            ("actualPrice", self.actual_price.map(|v| v.into_attr())),
            ("retailOff", self.retail_off.map(|v| v.into_attr())),
            ("breadcrumb", Some(self.breadcrumb.into_attr())),
            ("points", self.points.map(|v| v.into_attr())),
            ("createdAt", Some(self.created_at.into_attr())),
            ("updatedAt", Some(self.updated_at.into_attr())),
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
const INDEX_STATUS_CREATED_AT: SecondaryIndex = SecondaryIndex {
    name: "status-createdAt-index",
    hash_key: "status",
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

    pub async fn find_by_status(
        &self,
        status: Status,
        cursor: Option<Cursor>,
        limit: Option<i32>,
    ) -> AppResult<Vec<EntityWithCursor<Product>>> {
        let index = &INDEX_STATUS_CREATED_AT;

        query(
            self.cli
                .query()
                .table_name(self.table_name())
                .index_name(index.name)
                .key_conditions(index.hash_key, condition_eq(status.to_string().into_attr()))
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
