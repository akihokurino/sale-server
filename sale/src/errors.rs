use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Eq, PartialEq, derive_more::Display, Serialize, Deserialize)]
pub enum Kind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Internal,
}
impl Kind {
    pub fn with(self, msg: impl Into<String>) -> AppError {
        AppError {
            kind: self,
            msg: Some(msg.into()),
        }
    }

    pub fn withf<T>(self) -> impl FnOnce(T) -> AppError
    where
        T: Into<String>,
    {
        move |v| self.with(v)
    }

    pub fn from_src(self, src: impl std::error::Error + Send + Sync + 'static) -> AppError {
        AppError {
            kind: self,
            msg: Some(src.to_string()),
        }
    }

    pub fn from_srcf<T>(self) -> impl FnOnce(T) -> AppError
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        move |v| self.from_src(v)
    }
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub kind: Kind,
    pub msg: Option<String>,
}
impl derive_more::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.kind,
            self.msg
                .as_ref()
                .map(|v| format!(": {}", v))
                .unwrap_or_default(),
        )
    }
}
impl From<Kind> for AppError {
    fn from(kind: Kind) -> Self {
        Self { kind, msg: None }
    }
}

pub trait NotFoundToNone<T> {
    fn not_found_to_none(self) -> Result<Option<T>, AppError>;
}
impl<T> NotFoundToNone<T> for Result<T, AppError> {
    fn not_found_to_none(self) -> Result<Option<T>, AppError> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(v) => match v.kind {
                Kind::NotFound => Ok(None),
                _ => Err(v),
            },
        }
    }
}

macro_rules! impl_from_err_to_app_internal_err {
    ($T:ty) => {
        impl From<$T> for crate::errors::AppError {
            fn from(v: $T) -> Self {
                crate::errors::Kind::Internal.from_src(v)
            }
        }
    };
}
#[allow(unused)]
pub(crate) use impl_from_err_to_app_internal_err;
