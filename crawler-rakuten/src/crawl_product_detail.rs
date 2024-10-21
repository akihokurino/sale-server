use reqwest::Client;
use sale::domain::product::{Product, Source};
use sale::domain::time;
use sale::errors::Kind::Internal;
use sale::infra::aws::ddb::cursor::Cursor;
use sale::{di, AppResult};
use scraper::{Html, Selector};

pub async fn crawl(cursor: Option<Cursor>) -> AppResult<()> {
    let product_repo = di::DB_PRODUCT_REPOSITORY.get().await.clone();

    let products = product_repo
        .find_by_source(Source::Rakuten, cursor, Some(1))
        .await?;

    let mut cursor: Option<Cursor> = None;
    for product in products {
        match collect(product.entity.clone()).await {
            Ok(product) => {
                product_repo.put(product).await?;
            }
            Err(err) => {
                eprintln!(
                    "商品詳細のエラー: {:?}, 商品ID: {}",
                    err,
                    product.entity.id.as_str()
                );
            }
        }
        cursor = Some(product.cursor);
    }

    if let Some(cursor) = cursor {
        println!("next cursor: {}", cursor.to_string())
    }

    Ok(())
}

async fn collect(product: Product) -> AppResult<Product> {
    println!("商品詳細URL: {}", product.detail_url.as_str());
    let body = Client::new()
        .get("https://brandavenue.rakuten.co.jp/item/JD3262")
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

    let mut image_urls: Vec<url::Url> = vec![];
    let selector = Selector::parse("ul.item-images-list li.item-images-item img").unwrap();
    for element in document.select(&selector) {
        let mut image_url = element
            .value()
            .attr("src")
            .ok_or(Internal.with("画像URLが見つかりませんでした"))?
            .to_string();
        if image_url.starts_with("//") {
            image_url = format!("https:{}", image_url);
        }
        println!("画像URL: {}", image_url);
        image_urls.push(url::Url::parse(image_url.as_str()).map_err(Internal.from_srcf())?);
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

    let mut breadcrumb: Vec<String> = vec![];
    let selector = Selector::parse("ul.breadcrumb-list li.breadcrumb-item a").unwrap();
    for element in document.select(&selector) {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        println!("パンクズ: {}", v);
        breadcrumb.push(v);
    }

    Ok(product.update(
        Some(title),
        image_urls,
        Some(retail_price),
        Some(actual_price),
        Some(retail_off),
        breadcrumb,
        time::now(),
    ))
}
