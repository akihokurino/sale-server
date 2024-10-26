use crate::errors::AppError;
use crate::errors::Kind::Internal;
use crate::AppResult;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Adapter {
    client: Client,
}

impl Adapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // json api call
    pub async fn invoke<Req, Res>(&self, input: Req, arn: String) -> AppResult<Res>
    where
        Req: Serialize,
        Res: for<'a> Deserialize<'a>,
    {
        let json = serde_json::to_string(&input).map_err(Internal.from_srcf())?;
        let resp = self
            .client
            .invoke()
            .function_name(arn)
            .payload(Blob::new(json.into_bytes()))
            .send()
            .await
            .map_err(Internal.from_srcf())?;
        let payload = resp.payload.unwrap();
        let payload = String::from_utf8(payload.into_inner()).ok().unwrap();

        if let Some(error) = serde_json::from_str::<ErrorResponse>(&payload).ok() {
            return Err(AppError::deserialize(error.error_message));
        }
        let output = serde_json::from_str(&payload).map_err(Internal.from_srcf())?;

        Ok(output)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_message: String,
}
