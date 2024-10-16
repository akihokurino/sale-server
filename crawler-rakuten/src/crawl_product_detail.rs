use reqwest::Client;
use sale::errors::Kind::Internal;
use sale::AppResult;
use scraper::{Html, Selector};

pub async fn crawl() -> AppResult<()> {
    let url = "https://brandavenue.rakuten.co.jp/item/KX4493/";
    collect(url).await?;
    Ok(())
}

async fn collect(url: &str) -> AppResult<()> {
    let body = Client::new()
        .get(url)
        .send()
        .await
        .map_err(Internal.from_srcf())?
        .text()
        .await
        .map_err(Internal.from_srcf())?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".item-name").unwrap();
    let title = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("タイトルが見つかりませんでした"))?
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();
    println!("タイトル: {}", title);

    let selector = Selector::parse("ul.item-images-list li.item-images-item img").unwrap();
    for element in document.select(&selector) {
        let image_url = element
            .value()
            .attr("src")
            .ok_or(Internal.with("画像URLが見つかりませんでした"))?;
        println!("画像URL: {}", image_url);
    }

    let selector = Selector::parse(".item-price-retail-value").unwrap();
    let retail_price = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("定価が見つかりませんでした"))?
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();
    println!("元値: {}", retail_price);

    let selector = Selector::parse(".item-price-actual-value").unwrap();
    let actual_price = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("値段が見つかりませんでした"))?
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();
    println!("値段: {}", actual_price);

    let selector = Selector::parse(".item-price-retail-off").unwrap();
    let retail_off = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("割引率が見つかりませんでした"))?
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();
    println!("割引率: {}", retail_off);

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
