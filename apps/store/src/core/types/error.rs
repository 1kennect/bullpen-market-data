//! The error every route answers with, as a status and an error field.

/// Why a call did not happen. The body carries the message under error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The bearer token is absent or wrong.
    #[error("missing or invalid bearer token")]
    Unauthorized,
    /// The request names something the store will not accept.
    #[error("{0}")]
    Invalid(String),
    /// The row is not there for this tenant and user.
    #[error("{0}")]
    NotFound(&'static str),
    /// The row exists and belongs to another user, or the key is taken.
    #[error("{0}")]
    Conflict(&'static str),
    /// The database failed. The message never reaches the caller.
    #[error("store database: {0}")]
    Database(#[from] sqlx::Error),
}

impl Error {
    /// The HTTP status this error answers with.
    pub fn status(&self) -> u16 {
        match self {
            Self::Unauthorized => 401,
            Self::Invalid(_) => 400,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::Database(_) => 500,
        }
    }
}
