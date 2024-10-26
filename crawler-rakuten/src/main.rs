use anyhow::anyhow;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sale::errors::Kind::Internal;
use sale::infra::aws::ddb::cursor::Cursor;
use sale::infra::aws::lambda::types::crawler_rakuten::{
    CrawlDetailRequest, CrawlListRequest, Request, RequestBody,
};
use sale::infra::aws::lambda::types::sns::EventData;
use sale::{di, AppResult};
use serde_json::Value;
use std::env;

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
    let with_lambda = di::ENVIRONMENTS.with_lambda;

    match req.body {
        RequestBody::CrawlEntrypoint => {
            for url in LIST_URLS.into_iter() {
                sns.publish(
                    Request {
                        body: RequestBody::CrawlList(CrawlListRequest {
                            url: url.to_string(),
                        }),
                    },
                    sns_arn.clone(),
                )
                .await?;
            }
            Ok(())
        }
        RequestBody::CrawlList(body) => {
            let mut url = url::Url::parse(&body.url.clone()).map_err(Internal.from_srcf())?;
            if let Some(next_page) = crawl_product_list::crawl(&url).await? {
                if !with_lambda {
                    println!("ローカル実行により終了");
                    return Ok(());
                }

                if next_page <= 2 {
                    let query: Vec<(String, String)> = url
                        .query_pairs()
                        .filter(|(name, _)| name != "p")
                        .map(|(name, value)| (name.into_owned(), value.into_owned()))
                        .collect();
                    url.query_pairs_mut()
                        .clear()
                        .extend_pairs(&query)
                        .append_pair("p", &next_page.to_string());

                    sns.publish(
                        Request {
                            body: RequestBody::CrawlList(CrawlListRequest {
                                url: url.to_string(),
                            }),
                        },
                        sns_arn.clone(),
                    )
                    .await?;
                } else {
                    println!("{}がページ上限を超えました", url.as_str());
                }
            }

            Ok(())
        }
        RequestBody::CrawlDetail(body) => {
            let next_cursor = crawl_product_detail::crawl(
                body.cursor.map(|v| Cursor::from(v)),
                body.only_preparing,
            )
            .await?;
            if !with_lambda {
                println!("ローカル実行により終了 next_cursor: {:?}", next_cursor);
                return Ok(());
            }

            if let Some(cursor) = next_cursor {
                sns.publish(
                    Request {
                        body: RequestBody::CrawlDetail(CrawlDetailRequest {
                            cursor: Some(cursor),
                            only_preparing: body.only_preparing,
                        }),
                    },
                    sns_arn.clone(),
                )
                .await?;
            } else {
                println!("全ての商品詳細のクロールが完了しました");
            }
            Ok(())
        }
    }
}

const LIST_URLS: [&str; 1] = [
    "https://search.rakuten.co.jp/search/mall/-/551177/?f=13&p=1",
    //"https://search.rakuten.co.jp/search/mall/-/100371/?f=13&p=1",
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
