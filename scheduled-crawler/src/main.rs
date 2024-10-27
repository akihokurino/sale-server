use anyhow::anyhow;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sale::infra::aws::lambda;
use sale::infra::aws::lambda::types::sns::EventData;
use sale::{di, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    di::SSM_ADAPTER
        .get()
        .await
        .load_dotenv()
        .await
        .expect("failed to load ssm parameter store");

    let envs = di::ENVIRONMENTS.clone();

    if envs.with_lambda {
        let handler = service_fn(bridge);
        lambda_runtime::run(handler).await?;
    } else {
        let args: Vec<String> = env::args().collect();
        let json = args.get(1).expect("json params is required");
        let body: Request = serde_json::from_str(&json).unwrap();
        if let Err(err) = handler(body).await {
            eprintln!("error: {:?}", err);
            return Err(anyhow!(err).into());
        }
    }

    Ok(())
}

async fn bridge(event: LambdaEvent<Value>) -> Result<(), Error> {
    // SNS経由
    let data: Result<EventData, String> =
        serde_json::from_value(event.payload.clone()).map_err(|v| v.to_string());
    if let Ok(data) = data {
        let request: Request =
            serde_json::from_str(&data.records.first().unwrap().sns.message).unwrap();
        let result = handler(request).await;
        return match result {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("error: {:?}", err);
                Err(anyhow!(err).into())
            }
        };
    }

    // 直接Invoke経由
    let data: Result<Request, String> =
        serde_json::from_value(event.payload.clone()).map_err(|v| v.to_string());
    if let Ok(data) = data {
        let result = handler(data).await;
        return match result {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("error: {:?}", err);
                Err(anyhow!(err).into())
            }
        };
    }

    Err(anyhow!("不正なペイロードです").into())
}

async fn handler(req: Request) -> AppResult<()> {
    let sns_arn = di::ENVIRONMENTS.crawler_rakuten_sns_arn.clone();
    let sns = di::SNS_ADAPTER.get().await.clone();

    match req.command {
        Command::CrawlList => {
            sns.publish(
                lambda::types::crawler_rakuten::Request {
                    body: lambda::types::crawler_rakuten::RequestBody::CrawlEntrypoint,
                },
                sns_arn.clone(),
            )
            .await?;
            Ok(())
        }
        Command::CrawlDetail => {
            sns.publish(
                lambda::types::crawler_rakuten::Request {
                    body: lambda::types::crawler_rakuten::RequestBody::CrawlDetail(
                        lambda::types::crawler_rakuten::CrawlDetailRequest {
                            cursor: None,
                            only_preparing: false,
                        },
                    ),
                },
                sns_arn.clone(),
            )
            .await?;
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub command: Command,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    strum_macros::EnumString,
    strum_macros::Display,
    Serialize,
    Deserialize,
)]
pub enum Command {
    CrawlList,
    CrawlDetail,
}
