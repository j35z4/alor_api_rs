use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    #[serde(rename = "AccessToken")]
    pub access_token: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TypedBuilder)]
pub struct ResponseParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    pub migrate_to_chat_id: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    pub retry_after: Option<u16>,
}

