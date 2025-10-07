use serde::Serialize;
use thiserror::Error;

/// Custom error type for the application.
/// It implements `Serialize`, so it can be used in Tauri commands.
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum AppError {
    #[error(transparent)]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Database pool error: {0}")]
    PoolError(#[from] deadpool_diesel::PoolError),

    #[error("Database interaction error: {0}")]
    InteractError(#[from] deadpool_diesel::InteractError),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
