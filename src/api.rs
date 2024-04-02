use serde::{Deserialize, Serialize};

pub use alor_api_impl::*;

use crate::api_traits::ErrorResponse;

pub mod alor_api_impl;

pub static BASE_API_URL: &str = "https://api.alor.ru/md/v2/";
pub static BASE_REFRESH_TOKEN_URL: &str = "https://oauth.alor.ru/refresh";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, thiserror::Error)]
#[serde(untagged)]
pub enum Error {
    #[error("{0}")]
    Http(HttpError),
    #[error("Api Error {0:?}")]
    Api(ErrorResponse),
    #[error("Decode Error {0}")]
    Decode(String),
    #[error("Encode Error {0}")]
    Encode(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, thiserror::Error)]
#[error("Http Error {code}: {message}")]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}
