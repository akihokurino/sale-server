use crate::errors::AppError;

pub mod errors;

pub type AppResult<T> = Result<T, AppError>;
