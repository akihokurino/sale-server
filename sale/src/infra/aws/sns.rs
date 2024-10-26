use crate::errors::Kind::*;
use crate::AppResult;
use aws_sdk_sns::Client;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct Adapter {
    client: Client,
}

impl Adapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn publish<Req>(&self, input: Req, arn: String) -> AppResult<()>
    where
        Req: Serialize,
    {
        let json = serde_json::to_string(&input).map_err(Internal.from_srcf())?;
        self.client
            .publish()
            .topic_arn(&arn)
            .message(json)
            .send()
            .await
            .map_err(Internal.from_srcf())?;

        Ok(())
    }
}
