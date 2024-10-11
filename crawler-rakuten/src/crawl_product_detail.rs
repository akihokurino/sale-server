use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;

pub async fn crawl() -> Result<(), Box<dyn Error>> {
    let url = "https://brandavenue.rakuten.co.jp/item/KX4493/";

    collect(url).await?;

    Ok(())
}

async fn collect(url: &str) -> Result<(), Box<dyn Error>> {
    let body = Client::new().get(url).send().await?.text().await?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".item-name").unwrap();
    if let Some(element) = document.select(&selector).next() {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("タイトル: {}", v);
    }

    let selector = Selector::parse("ul.item-images-list li.item-images-item img").unwrap();
    for element in document.select(&selector) {
        let v = element.value().attr("src").unwrap_or("");
        println!("画像URL: {}", v);
    }

    let selector = Selector::parse(".item-price-retail-value").unwrap();
    if let Some(element) = document.select(&selector).next() {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("元値: {}", v);
    }

    let selector = Selector::parse(".item-price-actual-value").unwrap();
    if let Some(element) = document.select(&selector).next() {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("値段: {}", v);
    }

    let selector = Selector::parse(".item-price-retail-off").unwrap();
    if let Some(element) = document.select(&selector).next() {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("割引率: {}", v);
    }

    let selector = Selector::parse("ul.breadcrumb-list li.breadcrumb-item a").unwrap();
    for element in document.select(&selector) {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("パンクズ: {}", v);
    }

    Ok(())
}
