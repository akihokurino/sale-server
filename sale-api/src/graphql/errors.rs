use async_graphql::{ErrorExtensions, FieldError};
use derive_more::From;
use sale::errors::AppError;
use sale::errors::Kind::*;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(From)]
pub enum Error {
    App(AppError),
    Field(FieldError),
}
impl Into<FieldError> for Error {
    fn into(self) -> FieldError {
        let err = match self {
            Error::App(v) => v,
            Error::Field(v) => Internal.from_src(FieldStdErr(v)),
        };

        let mut ferr = FieldError::new(match err.kind {
            BadRequest => err
                .msg
                .clone()
                .unwrap_or_else(|| "入力内容に誤りがあります".into()),
            Unauthorized => err
                .msg
                .clone()
                .unwrap_or_else(|| "認証されていません".into()),
            Forbidden => err
                .msg
                .clone()
                .unwrap_or_else(|| "許可されていないアクションです".into()),
            NotFound => err
                .msg
                .clone()
                .unwrap_or_else(|| "指定されたリソースが見つかりません".into()),
            Internal => "内部エラーが発生しました".into(), // 内部エラーはユーザーには詳細を伏せる
        })
        .extend_with(|_, ext| {
            ext.set(
                "code",
                match err.kind {
                    BadRequest => "BAD_REQUEST",
                    Unauthorized => "UNAUTHORIZED",
                    Forbidden => "FORBIDDEN",
                    NotFound => "NOT_FOUND",
                    Internal => "INTERNAL",
                }
                .to_string(),
            )
        });
        ferr.source = Some(Arc::new(err));
        ferr
    }
}

// FieldErrorがstd::errorを実装していないため
#[derive(Debug)]
struct FieldStdErr(FieldError);
impl Display for FieldStdErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message)
    }
}
impl std::error::Error for FieldStdErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(v) = &self.0.source {
            v.downcast_ref::<Arc<dyn std::error::Error>>()
                .map(|v| v.as_ref())
        } else {
            None
        }
    }
}
