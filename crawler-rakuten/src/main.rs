use anyhow::anyhow;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sale::{di, AppResult};
use serde::{Deserialize, Serialize};

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

    let handler = service_fn(handler);
    lambda_runtime::run(handler).await?;

    Ok(())
}

async fn handler(event: LambdaEvent<Request>) -> Result<(), Error> {
    let (request, _context) = event.into_parts();
    let result = _handler(request.task).await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(err).into()),
    }
}

async fn _handler(task: Task) -> AppResult<()> {
    match task {
        Task::CrawlList => crawl_product_list::crawl().await,
        Task::CrawlDetail => crawl_product_detail::crawl().await,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub task: Task,
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
