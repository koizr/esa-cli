use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {}, message: {}", .0.error, .0.message)]
    ApiError(ErrorResponse),

    #[error("HTTP Error: {}", 0)]
    HttpError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}
