use anyhow::anyhow;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sale::errors::Kind::Internal;
use sale::{di, AppResult};
use serde::{Deserialize, Serialize};
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
        let handler = service_fn(handler);
        lambda_runtime::run(handler).await?;
    } else {
        let args: Vec<String> = env::args().collect();
        let task = args.get(1).expect("task is required");
        let task = Task::from_str(task).expect("invalid task");
        let url = args.get(2).expect("url is required").to_string();
        if let Err(err) = _handler(Request { task, url }).await {
            eprintln!("error: {:?}", err);
            return Err(anyhow!(err).into());
        }
    }

    Ok(())
}

async fn handler(event: LambdaEvent<Request>) -> Result<(), Error> {
    let (request, _context) = event.into_parts();
    let result = _handler(request).await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            eprintln!("error: {:?}", err);
            Err(anyhow!(err).into())
        }
    }
}

async fn _handler(req: Request) -> AppResult<()> {
    let url = url::Url::parse(&req.url).map_err(Internal.from_srcf())?;

    match req.task {
        Task::CrawlList => crawl_product_list::crawl(url).await,
        Task::CrawlDetail => crawl_product_detail::crawl().await,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub task: Task,
    pub url: String,
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
    CrawlList,
    CrawlDetail,
}
