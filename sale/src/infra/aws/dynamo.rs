use crate::domain::Id;
use aws_sdk_dynamodb::types::AttributeValue;
use std::marker::PhantomData;

mod errors;
pub mod types;

trait MustPresent<T> {
    fn must_present(self) -> Result<T, String>;
}
impl<T> MustPresent<T> for Option<T> {
    fn must_present(self) -> Result<T, String> {
        self.ok_or_else(|| "missing field".to_string())
    }
}

trait AttrConv {
    fn must_s(self) -> Result<String, String>;
    fn from_s(s: &str) -> AttributeValue;
}
impl AttrConv for AttributeValue {
    fn must_s(self) -> Result<String, String> {
        self.as_s()
            .map(|v| v.to_string())
            .map_err(move |_| "cannot convert".to_string())
    }

    fn from_s(s: &str) -> AttributeValue {
        AttributeValue::S(s.to_string())
    }
}

impl<E> Id<E> {
    pub fn typename() -> &'static str {
        std::any::type_name::<E>()
    }
}

pub(crate) trait HasTableName {
    fn table_name() -> String;
}

#[derive(Clone, Debug)]
pub(crate) struct TableNameProvider {
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
    pub(crate) cli: aws_sdk_dynamodb::Client,
    table_name_provider: TableNameProvider,
    _phantom: PhantomData<fn() -> E>,
}
impl<E: HasTableName> TableRepository<E> {
    fn table_name(&self) -> String {
        self.table_name_provider.get(E::table_name().as_str())
    }
}
impl<E> TableRepository<E> {
    pub(crate) fn new(
        cli: aws_sdk_dynamodb::Client,
        table_name_provider: TableNameProvider,
    ) -> Self {
        Self {
            cli,
            table_name_provider,
            _phantom: PhantomData,
        }
    }
}
