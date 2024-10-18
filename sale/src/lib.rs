use crate::errors::AppError;

pub mod di;
pub mod domain;
pub mod env;
pub mod errors;
pub mod infra;
mod sync;

pub type AppResult<T> = Result<T, AppError>;
