use std::error::Error;

mod crawl_product_detail;
mod crawl_product_list;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    crawl_product_detail::crawl().await?;

    Ok(())
}
