pub mod handler;
pub mod jwt;
pub mod error;

use error::WebAppError;
pub type Result<T> = anyhow::Result<T , WebAppError>;
