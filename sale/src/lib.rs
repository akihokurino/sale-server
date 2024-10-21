use crate::errors::AppError;

pub mod di;
pub mod domain;
pub mod env;
pub mod errors;
pub mod infra;
mod sync;

pub type AppResult<T> = Result<T, AppError>;

trait MustPresent<T> {
    fn must_present(self) -> Result<T, String>;
}
impl<T> MustPresent<T> for Option<T> {
    fn must_present(self) -> Result<T, String> {
        self.ok_or_else(|| "missing value".to_string())
    }
}
