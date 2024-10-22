use crate::domain::time::LocalDateTime;
use crate::domain::Id;
use crate::infra::aws::ddb::{anchor_attr_value, HasTypeName};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{Local, TimeZone};
use std::collections::HashMap;
use std::str::FromStr;

pub mod product;

pub trait ToAttrValue {
    fn into_attr(self) -> AttributeValue;
}
impl ToAttrValue for String {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self)
    }
}
impl ToAttrValue for &str {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::S(self.to_string())
    }
}
impl ToAttrValue for Vec<String> {
    fn into_attr(self) -> AttributeValue {
        AttributeValue::L(self.into_iter().map(|v| v.into_attr()).collect())
    }
}
impl ToAttrValue for LocalDateTime {
    fn into_attr(self) -> AttributeValue {
        let v = self.timestamp_nanos_opt().unwrap();
        AttributeValue::N(v.to_string())
    }
}

pub trait FromAttrValue {
    fn to_s(self) -> Result<String, String>;
    fn to_s_list(self) -> Result<Vec<String>, String>;
    fn to_date_time(self) -> Result<LocalDateTime, String>;
}
impl FromAttrValue for AttributeValue {
    fn to_s(self) -> Result<String, String> {
        self.as_s()
            .map(|v| v.to_string())
            .map_err(|_| "cannot convert string".to_string())
    }

    fn to_s_list(self) -> Result<Vec<String>, String> {
        let v = self
            .as_l()
            .map_err(|_| "cannot convert string list".to_string())?
            .clone();
        v.into_iter()
            .map(|v| v.to_s())
            .collect::<Result<Vec<String>, String>>()
    }

    fn to_date_time(self) -> Result<LocalDateTime, String> {
        let v = self
            .as_n()
            .map_err(|_| "cannot convert timestamp".to_string())?
            .clone();
        Ok(Local.timestamp_nanos(i64::from_str(&v).map_err(|_| "invalid timestamp")?))
    }
}

impl<E: HasTypeName> Into<AttributeValue> for Id<E> {
    fn into(self) -> AttributeValue {
        AttributeValue::S(format!("{}#{}", E::type_name(), self.as_str()))
    }
}
impl<E: HasTypeName> TryFrom<AttributeValue> for Id<E> {
    type Error = String;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        let v = value.to_s()?;
        let v = v
            .strip_prefix(&format!("{}#", E::type_name()))
            .ok_or_else(|| "invalid id".to_string())?;
        Ok(Self::new(v))
    }
}
impl<E: HasTypeName> Id<E> {
    fn into_attr_map(self) -> HashMap<String, AttributeValue> {
        [("pk", Some(self.into())), ("sk", Some(anchor_attr_value()))]
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone().unwrap()))
            .collect()
    }
}
