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

trait FromAttr {
    fn to_s(self) -> Result<String, String>;
}
impl FromAttr for AttributeValue {
    fn to_s(self) -> Result<String, String> {
        self.as_s()
            .map(|v| v.to_string())
            .map_err(move |_| "cannot convert".to_string())
    }
}

trait ToAttr {
    fn into_attr(self) -> AttributeValue;
}
impl ToAttr for String {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self)
    }
}
impl ToAttr for &str {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self.to_string())
    }
}

impl<E> Id<E> {
    pub fn typename() -> &'static str {
        std::any::type_name::<E>()
    }

    pub fn sk() -> &'static str {
        "#"
    }
}
impl<E> Into<AttributeValue> for Id<E> {
    fn into(self) -> AttributeValue {
        AttributeValue::S(format!("{}#{}", Self::typename(), self.as_str()))
    }
}
impl<E> TryFrom<AttributeValue> for Id<E> {
    type Error = String;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        let v = value.to_s()?;
        let v = v
            .strip_prefix(Self::typename())
            .ok_or_else(|| "invalid id".to_string())?;
        Ok(Self::new(v))
    }
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

pub trait HasTableName {
    fn table_name() -> String;
}

#[derive(Clone, Debug)]
pub struct TableRepository<E> {
    cli: aws_sdk_dynamodb::Client,
    table_name_provider: TableNameProvider,
    _phantom: PhantomData<fn() -> E>,
}
impl<E: HasTableName> TableRepository<E> {
    fn table_name(&self) -> String {
        self.table_name_provider.get(E::table_name().as_str())
    }
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
