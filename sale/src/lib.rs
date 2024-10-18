use crate::errors::AppError;

pub mod domain;
pub mod errors;
pub mod infra;

pub type AppResult<T> = Result<T, AppError>;
