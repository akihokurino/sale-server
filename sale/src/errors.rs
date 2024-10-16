use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display, Serialize, Deserialize)]
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
impl Display for AppError {
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
