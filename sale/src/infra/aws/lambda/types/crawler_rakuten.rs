use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub body: RequestBody,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestBody {
    CrawlEntrypoint,
    CrawlList(CrawlListRequest),
    CrawlDetail(CrawlDetailRequest),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlListRequest {
    pub url: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlDetailRequest {
    pub cursor: Option<String>,
    pub only_preparing: bool,
}
