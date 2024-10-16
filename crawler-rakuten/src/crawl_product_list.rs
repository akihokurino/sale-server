use reqwest::Client;
use sale::errors::Kind::Internal;
use sale::AppResult;
use scraper::{Html, Selector};
use tokio::time::{sleep, Duration};

pub async fn crawl() -> AppResult<()> {
    let mut page = 1;
    loop {
        let url = format!(
            "https://search.rakuten.co.jp/search/mall/-/551177/?f=13&p={}",
            page
        );
        println!("--------------------------------------------------------------------");
        println!("ページ: {}", url);
        println!("--------------------------------------------------------------------");

        let products = collect(&url).await?;
        for product in products {
            println!("商品URL: {}, ポイント: {}", product.url, product.points);
        }

        sleep(Duration::from_secs(3)).await;
        page += 1;

        if page > 1000 {
            break;
        }
    }

    Ok(())
}

async fn collect(url: &str) -> AppResult<Vec<Product>> {
    let body = Client::new()
        .get(url)
        .send()
        .await
        .map_err(Internal.from_srcf())?
        .text()
        .await
        .map_err(Internal.from_srcf())?;
    let document = Html::parse_document(&body);

    let item_selector = Selector::parse(".searchresultitems .searchresultitem").unwrap();
    let url_selector = Selector::parse(".image-link-wrapper--3P6dv").unwrap();
    let points_selector = Selector::parse(".points--AHzKn span").unwrap();

    let mut products: Vec<Product> = vec![];
    for element in document.select(&item_selector) {
        let e_ref = element
            .select(&url_selector)
            .next()
            .ok_or(Internal.with("URLが見つかりませんでした"))?;
        let url = e_ref
            .value()
            .attr("href")
            .ok_or(Internal.with("URLが見つかりませんでした"))?
            .to_string();

        let e_ref = element
            .select(&points_selector)
            .next()
            .ok_or(Internal.with("ポイントが見つかりませんでした"))?;
        let points = e_ref.text().collect::<Vec<_>>().concat().trim().to_string();

        if !url.is_empty() {
            products.push(Product { url, points });
        }
    }

    Ok(products)
}

#[derive(Debug, Clone)]
struct Product {
    url: String,
    points: String,
}
