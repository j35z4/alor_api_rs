use serde::{Deserialize, Serialize};

pub use alor_api::*;

pub mod alor_api;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MethodResponse<T> {
    pub ok: bool,
    pub result: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub ok: bool,
    pub description: String,
    pub error_code: u64,
}
