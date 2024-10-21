use crate::infra::aws::ddb::cursor::EntityWithCursor;
use crate::infra::aws::ddb::types::{FromAttr, ToAttr};
use aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder;
use aws_sdk_dynamodb::types::{AttributeValue, ComparisonOperator, Condition, Select};
use derive_more::Into;
use std::collections::HashMap;
use std::marker::PhantomData;

pub mod cursor;
mod errors;
mod index;
pub mod types;

pub trait HasTableName {
    fn table_name() -> String;
}

pub trait HasTypeName {
    fn type_name() -> String;
}

#[derive(Clone, Debug)]
pub struct TableNameProvider {
    prefix: String,
}
impl TableNameProvider {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    pub fn get(&self, basename: &str) -> String {
        format!("{}{}", self.prefix, basename)
    }
}

#[derive(Clone, Debug)]
pub struct TableRepository<E> {
    cli: aws_sdk_dynamodb::Client,
    table_name_provider: TableNameProvider,
    _phantom: PhantomData<fn() -> E>,
}
impl<E> TableRepository<E> {
    pub fn new(cli: aws_sdk_dynamodb::Client, table_name_provider: TableNameProvider) -> Self {
        Self {
            cli,
            table_name_provider,
            _phantom: PhantomData,
        }
    }
}
impl<E: HasTableName> TableRepository<E> {
    fn table_name(&self) -> String {
        self.table_name_provider.get(E::table_name().as_str())
    }
}

// https://docs.aws.amazon.com/ja_jp/amazondynamodb/latest/developerguide/Query.Pagination.html
// データ量によって分割した結果が返ってくるので、limitに達していないがhas_nextがtrueになることがある
async fn query<T>(
    q: QueryFluentBuilder,
    limit: Option<i32>,
    conv: impl Fn(HashMap<String, AttributeValue>) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let mut limit = limit;
    let mut q = q.set_limit(limit);

    let mut items: Vec<T> = vec![];
    while {
        let query_res = q.clone().send().await.unwrap();
        let mut partial_items = query_res
            .items
            .ok_or_else(|| "result items missing")?
            .into_iter()
            .map(|v| conv(v))
            .collect::<Result<Vec<_>, _>>()?;

        let has_next = query_res.last_evaluated_key.is_some();
        limit = limit.map(|limit| limit - partial_items.len() as i32);
        items.append(&mut partial_items);
        q = q
            .set_exclusive_start_key(query_res.last_evaluated_key)
            .set_limit(limit);
        limit.map(|v| v > 0).unwrap_or(true) && has_next
    } {}

    Ok(items)
}

// https://docs.aws.amazon.com/ja_jp/amazondynamodb/latest/developerguide/Query.Pagination.html
// データ量によって分割した結果が返ってくるので、whileでhas_nextを見る必要がある
async fn count(q: QueryFluentBuilder) -> Result<usize, String> {
    let mut q = q.select(Select::Count);

    let mut count: usize = 0;
    let mut has_next = true;
    while has_next {
        let query_res = q.clone().send().await.unwrap();
        count += query_res.count as usize;
        has_next = query_res.last_evaluated_key.is_some();
        q = q.set_exclusive_start_key(query_res.last_evaluated_key);
    }
    Ok(count)
}

#[allow(unused)]
fn anchor_attr_value() -> AttributeValue {
    AttributeValue::S("#".into())
}

#[allow(unused)]
fn condition_sk_type<T: HasTypeName>() -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::BeginsWith)
        .attribute_value_list(AttributeValue::S(format!("{}#", T::type_name())))
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_eq(v: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Eq)
        .attribute_value_list(v)
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_gt(v: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Gt)
        .attribute_value_list(v)
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_ge(v: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Ge)
        .attribute_value_list(v)
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_lt(v: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Lt)
        .attribute_value_list(v)
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_le(v: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Le)
        .attribute_value_list(v)
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_contains(v: impl Into<String>) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Contains)
        .attribute_value_list(v.into().into_attr())
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_begins_with(v: impl Into<String>) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::BeginsWith)
        .attribute_value_list(v.into().into_attr())
        .build()
        .unwrap()
}
#[allow(unused)]
fn condition_between(a: AttributeValue, b: AttributeValue) -> Condition {
    Condition::builder()
        .comparison_operator(ComparisonOperator::Between)
        .attribute_value_list(a)
        .attribute_value_list(b)
        .build()
        .unwrap()
}
