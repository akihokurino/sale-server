use anyhow::anyhow;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sale::errors::Kind::Internal;
use sale::infra::aws::ddb::cursor::Cursor;
use sale::{di, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::str::FromStr;

mod crawl_product_detail;
mod crawl_product_list;

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
        let task = args.get(1).expect("task is required");
        let task = Task::from_str(task).expect("invalid task");
        let url =
            args.get(2)
                .map(|v| v.to_string())
                .and_then(|v| if v.is_empty() { None } else { Some(v) });
        let cursor =
            args.get(3)
                .map(|v| v.to_string())
                .and_then(|v| if v.is_empty() { None } else { Some(v) });
        if let Err(err) = handler(Request { task, url, cursor }).await {
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

    println!(
        "Crawler Rakuten Req: {:?}",
        serde_json::to_string(&req).unwrap()
    );

    match req.task {
        Task::CrawlEntrypoint => {
            for url in LIST_URLS.into_iter() {
                sns.publish(
                    Request {
                        task: Task::CrawlList,
                        url: Some(url.to_string()),
                        cursor: None,
                    },
                    sns_arn.clone(),
                )
                .await?;
            }
            Ok(())
        }
        Task::CrawlList => {
            let url = url::Url::parse(&req.url.unwrap()).map_err(Internal.from_srcf())?;
            crawl_product_list::crawl(url).await
        }
        Task::CrawlDetail => crawl_product_detail::crawl(req.cursor.map(|v| Cursor::from(v))).await,
    }
}

#[derive(Serialize, Deserialize)]
struct EventData {
    #[serde(rename = "Records")]
    pub records: Vec<Record>,
}
#[derive(Serialize, Deserialize)]
struct Record {
    #[serde(rename = "Sns")]
    pub sns: Sns,
}
#[derive(Serialize, Deserialize)]
struct Sns {
    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub task: Task,
    pub url: Option<String>,
    pub cursor: Option<String>,
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
pub enum Task {
    CrawlEntrypoint,
    CrawlList,
    CrawlDetail,
}

const LIST_URLS: [&str; 2] = [
    "https://search.rakuten.co.jp/search/mall/-/551177/?f=13&p=1",
    "https://search.rakuten.co.jp/search/mall/-/100371/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/558885/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/216131/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/216129/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/558929/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100433/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100533/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100227/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/551167/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100316/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100317/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/510915/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/510901/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100026/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/564500/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/211742/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/562637/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/565004/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101240/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/112493/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101070/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101077/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100804/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/215783/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/558944/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100005/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101213/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100938/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/551169/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/100939/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/566382/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101205/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/200162/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101114/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/503190/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101438/?f=13&p=1",
    // "https://search.rakuten.co.jp/search/mall/-/101381/?f=13&p=1",
];
