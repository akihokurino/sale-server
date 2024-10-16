use crate::errors::AppError;

mod domain;
pub mod errors;

pub type AppResult<T> = Result<T, AppError>;
