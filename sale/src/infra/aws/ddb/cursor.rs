use aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder;
use aws_sdk_dynamodb::types::AttributeValue;
use base64::Engine;
use derive_more::{AsRef, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Into, From, Display, AsRef, Ord, PartialOrd, Eq, PartialEq)]
pub struct Cursor(pub String);
impl TryInto<HashMap<String, AttributeValue>> for Cursor {
    type Error = String;
    fn try_into(self) -> Result<HashMap<String, AttributeValue>, Self::Error> {
        let v = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(self.0)
            .map_err(|v| format!("failed to decode cursor: {}", v))?;
        let v = serde_json::from_slice::<HashMap<String, _CursorInnerValue>>(&v)
            .map_err(|v| format!("failed to decode cursor: {}", v))?;
        v.into_iter()
            .map(|(k, v)| Ok((k, v.try_into()?)))
            .collect::<Result<HashMap<String, AttributeValue>, String>>()
    }
}
impl TryFrom<HashMap<String, AttributeValue>> for Cursor {
    type Error = String;
    fn try_from(v: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let v = v
            .into_iter()
            .map(|(k, v)| v.try_into().map(|v| (k, v)))
            .collect::<Result<HashMap<String, _CursorInnerValue>, _>>()?;
        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&v).unwrap())
            .into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum _CursorInnerValue {
    S(String),
    N(String),
}
impl TryInto<AttributeValue> for _CursorInnerValue {
    type Error = String;
    fn try_into(self) -> Result<AttributeValue, Self::Error> {
        Ok(match self {
            Self::S(v) => AttributeValue::S(v),
            Self::N(v) => AttributeValue::N(v),
        })
    }
}
impl TryFrom<AttributeValue> for _CursorInnerValue {
    type Error = String;
    fn try_from(v: AttributeValue) -> Result<Self, Self::Error> {
        Ok(match v {
            AttributeValue::S(v) => Self::S(v),
            AttributeValue::N(v) => Self::N(v),
            _ => return Err("invalid cursor value type".to_string()),
        })
    }
}

pub trait WithCursor {
    fn with_cursor(self, cursor: Option<Cursor>) -> Result<QueryFluentBuilder, String>;
}
impl WithCursor for QueryFluentBuilder {
    fn with_cursor(self, cursor: Option<Cursor>) -> Result<QueryFluentBuilder, String> {
        Ok(match cursor {
            None => self,
            Some(c) => self.set_exclusive_start_key(Some(
                c.try_into().map_err(|_| "カーソルの書式が不正です")?,
            )),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EntityWithCursor<E> {
    pub entity: E,
    pub cursor: Cursor,
}
impl<E> EntityWithCursor<E> {
    pub fn new(entity: E, cursor: Cursor) -> Self {
        Self { entity, cursor }
    }
}
pub fn entity_with_cursor_conv_from<T: 'static>(
    evaluated_key_names: Vec<&'static str>,
    conv: fn(HashMap<String, AttributeValue>) -> Result<T, String>,
) -> Box<dyn Fn(HashMap<String, AttributeValue>) -> Result<EntityWithCursor<T>, String> + Send + Sync>
{
    Box::new(move |attrs: HashMap<String, AttributeValue>| {
        let cursor = {
            let mut attrs = attrs.clone();
            let key_map = evaluated_key_names
                .iter()
                .map(|k| {
                    attrs
                        .remove(k.to_owned())
                        .ok_or_else(|| format!("missing key: {}", k))
                        .map(|v| (k.to_string(), v))
                })
                .collect::<Result<HashMap<String, AttributeValue>, String>>()?;

            key_map.try_into()?
        };

        Ok(EntityWithCursor::new(conv(attrs)?, cursor))
    })
}
