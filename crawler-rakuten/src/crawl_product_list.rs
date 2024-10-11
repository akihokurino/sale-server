use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;
use tokio::time::{sleep, Duration};

pub async fn crawl() -> Result<(), Box<dyn Error>> {
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

async fn collect(url: &str) -> Result<Vec<Product>, Box<dyn Error>> {
    let body = Client::new().get(url).send().await?.text().await?;
    let document = Html::parse_document(&body);

    let item_selector = Selector::parse(".searchresultitems .searchresultitem").unwrap();
    let url_selector = Selector::parse(".image-link-wrapper--3P6dv").unwrap();
    let points_selector = Selector::parse(".points--AHzKn span").unwrap();

    let mut products: Vec<Product> = vec![];
    for element in document.select(&item_selector) {
        let url = element
            .select(&url_selector)
            .next()
            .map(|e| e.value().attr("href").unwrap_or("").to_string())
            .unwrap_or_default();

        let points = element
            .select(&points_selector)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().concat().trim().to_string())
            .unwrap_or_default();

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
