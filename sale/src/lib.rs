use crate::errors::AppError;

mod domain;
pub mod errors;
mod infra;

pub type AppResult<T> = Result<T, AppError>;
