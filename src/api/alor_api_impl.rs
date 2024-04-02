use std::time::Duration;

use typed_builder::TypedBuilder;
use ureq::Response;

use crate::{BASE_API_URL, BASE_REFRESH_TOKEN_URL};
use crate::api_params::TokenResponse;
use crate::api_traits::AlorApi;
use crate::api_traits::ErrorResponse;

use super::Error;
use super::HttpError;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Api {
    #[builder(setter(into))]
    pub api_url: String,
    #[builder(setter(into))]
    pub refresh_token_url: String,
    #[builder(setter(into))]
    pub refresh_token: String,
    #[builder(setter(into), default)]
    pub access_token: Option<String>,
    #[builder(default_code = "ureq::builder().timeout(Duration::from_secs(500)).build()")]
    pub request_agent: ureq::Agent,
}

impl Api {
    pub fn update_access_token(&self) -> Result<String, Error> {
        let url = format!(
            "{}?token={}",
            &self.refresh_token_url,
            &self.refresh_token
        );
        let response: Response = ureq::post(&url)
            .set("accept", "application/json")
            .set("Content-Length", "0")
            .call()?;

        let text = response.into_string()?;
        let token_response: TokenResponse = serde_json::from_str(&text)?;
        Ok(token_response.access_token)
    }

    pub fn new(refresh_token: String) -> Self {
        Self::builder()
            .api_url(BASE_API_URL)
            .refresh_token_url(BASE_REFRESH_TOKEN_URL)
            .refresh_token(refresh_token)
            .build()
    }

    pub fn new_url(api_url: String, refresh_token_url: String, refresh_token: String) -> Self {
        Self::builder()
            .api_url(api_url)
            .refresh_token_url(refresh_token_url)
            .refresh_token(refresh_token)
            .build()
    }

    pub fn encode_params<T: serde::ser::Serialize + std::fmt::Debug>(
        params: &T,
    ) -> Result<String, Error> {
        serde_json::to_string(params).map_err(|e| Error::Encode(format!("{e:?} : {params:?}")))
    }

    pub fn decode_response<T: serde::de::DeserializeOwned>(response: Response) -> Result<T, Error> {
        match response.into_string() {
            Ok(message) => {
                let json_result: Result<T, serde_json::Error> = serde_json::from_str(&message);

                match json_result {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        let err = Error::Decode(format!("{e:?} : {message:?}"));
                        Err(err)
                    }
                }
            }
            Err(e) => {
                let err = Error::Decode(format!("Failed to decode response: {e:?}"));
                Err(err)
            }
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Status(code, response) => match response.into_string() {
                Ok(message) => {
                    let json_result: Result<ErrorResponse, serde_json::Error> =
                        serde_json::from_str(&message);

                    match json_result {
                        Ok(result) => Self::Api(result),
                        Err(_) => {
                            let error = HttpError { code, message };
                            Self::Http(error)
                        }
                    }
                }
                Err(_) => {
                    let message = "Failed to decode response".to_string();
                    let error = HttpError { code, message };
                    Self::Http(error)
                }
            },
            ureq::Error::Transport(transport_error) => {
                let message = format!("{transport_error:?}");
                let error = HttpError { message, code: 500 };
                Self::Http(error)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{}", error);
        let error = HttpError { code: 500, message };
        Self::Http(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        let message = format!("{}", error);
        let error = HttpError { code: 500, message };
        Self::Http(error)
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        let message = format!("{}", error);
        let error = HttpError { code: 500, message };
        Self::Http(error)
    }
}

impl AlorApi for Api {
    type Error = Error;

    fn post<T1: serde::ser::Serialize + std::fmt::Debug, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Error> {
        let url = format!("{}/{method}", self.api_url);
        let prepared_request = self
            .request_agent
            .post(&url)
            .set("Content-Type", "application/json");

        let response = match params {
            None => prepared_request.call()?,
            Some(data) => {
                let json = Self::encode_params(&data)?;

                prepared_request.send_string(&json)?
            }
        };

        let parsed_response: T2 = Self::decode_response(response)?;

        Ok(parsed_response)
    }

    fn get<T1: serde::ser::Serialize + std::fmt::Debug, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Error> {
        let mut url = format!("{}/{}", self.api_url, method);

        if let Some(data) = params {
            let params_string = serde_urlencoded::to_string(data)?;
            url = format!("{}?{}", url, params_string);
        }

        let response = self
            .request_agent
            .get(&url)
            .set("Content-Type", "application/json")
            .call()?;

        let parsed_response: T2 = Self::decode_response(response)?;

        Ok(parsed_response)
    }
}


#[cfg(test)]
mod tests {
    use crate::Api;

    #[test]
    fn test_update_access_token() {
        let response_string = "{\"AccessToken\":\"new_token\"}";
        let url_path = "/refresh";
        let some_refresh_token = "some_refresh_token".to_string();
        let check_string = "new_token".to_string();

        let mut server = mockito::Server::new();
        let _m = server
            .mock("POST", format!("{}?token={}", url_path, some_refresh_token).as_str())
            .with_status(200)
            .with_body(response_string)
            .create();
        let mock_url = server.url();
        let mock_refresh_url = format!("{}{}", mock_url, url_path);
        let api = Api::new_url(mock_url, mock_refresh_url, some_refresh_token);

        let response = api.update_access_token();

        match response {
            Ok(token) => {
                assert_eq!(token, check_string);
            }
            Err(e) => {
                println!("Error occurred: {:?}", e);
            }
        }
    }
}
