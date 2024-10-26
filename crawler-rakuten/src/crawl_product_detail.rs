use encoding_rs::EUC_JP;
use encoding_rs_io::DecodeReaderBytesBuilder;
use reqwest::Client;
use sale::domain::product::{Product, Source};
use sale::domain::time;
use sale::errors::Kind::Internal;
use sale::infra::aws::ddb::cursor::Cursor;
use sale::{di, AppResult};
use scraper::{Html, Selector};
use std::io::Read;
use std::time::Duration;
use tokio::time::sleep;

pub async fn crawl(cursor: Option<Cursor>) -> AppResult<Option<String>> {
    let product_repo = di::DB_PRODUCT_REPOSITORY.get().await.clone();

    let products = product_repo
        .find_by_source(Source::Rakuten, cursor.clone(), Some(10))
        .await?;
    if products.is_empty() {
        return Ok(None);
    }

    println!(
        "詳細クロールする商品数: {}, cursor: {}",
        products.len(),
        cursor.map(|v| v.to_string()).unwrap_or_default()
    );

    let mut cursor: Option<Cursor> = None;
    for product in products {
        if detect_brandavenue(product.entity.detail_url.clone()) {
            match collect_brandavenue(product.entity.clone()).await {
                Ok(product) => {
                    product_repo.put(product).await?;
                }
                Err(err) => {
                    eprintln!(
                        "[brandavenue] 商品詳細のエラー: {:?}, 商品ID: {}",
                        err,
                        product.entity.id.as_str()
                    );
                }
            }
        } else {
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
        }

        cursor = Some(product.cursor);

        sleep(Duration::from_secs(1)).await;
    }

    Ok(cursor.map(|v| v.to_string()))
}

fn detect_brandavenue(url: url::Url) -> bool {
    url.host_str() == Some("brandavenue.rakuten.co.jp")
        || url.path_segments().map_or(false, |mut segments| {
            segments.any(|s| s.contains("stylife"))
        })
}

// "https://brandavenue.rakuten.co.jp/item/JD3262"
async fn collect_brandavenue(product: Product) -> AppResult<Product> {
    println!("[brandavenue] 商品詳細URL: {}", product.detail_url.as_str());
    let body = Client::new()
        .get(product.detail_url.as_str())
        .send()
        .await
        .map_err(Internal.from_srcf())?
        .text()
        .await
        .map_err(Internal.from_srcf())?;
    let document = Html::parse_document(&body);

    // let mut file = File::create("test.html").map_err(Internal.from_srcf())?;
    // file.write_all(body.as_bytes())
    //     .map_err(Internal.from_srcf())?;

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
        image_urls.push(url::Url::parse(image_url.as_str()).map_err(Internal.from_srcf())?);
    }

    let selector = Selector::parse(".item-price-retail-value").unwrap();
    let retail_price = document
        .select(&selector)
        .next()
        .map(|v| v.text().collect::<Vec<_>>().concat().trim().to_string());

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

    let selector = Selector::parse(".item-price-retail-off").unwrap();
    let retail_off = document
        .select(&selector)
        .next()
        .map(|v| v.text().collect::<Vec<_>>().concat().trim().to_string());

    let mut breadcrumb: Vec<String> = vec![];
    let selector = Selector::parse("ul.breadcrumb-list li.breadcrumb-item a").unwrap();
    for element in document.select(&selector) {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        breadcrumb.push(v);
    }

    Ok(product.update(
        Some(title),
        image_urls,
        retail_price,
        Some(actual_price),
        retail_off,
        breadcrumb,
        time::now(),
    ))
}

async fn collect(product: Product) -> AppResult<Product> {
    println!("商品詳細URL: {}", product.detail_url.as_str());
    let response = Client::new()
        .get(product.detail_url.as_str())
        .send()
        .await
        .map_err(Internal.from_srcf())?;
    let bytes = response.bytes().await.map_err(Internal.from_srcf())?;
    let mut decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(EUC_JP))
        .build(bytes.as_ref());
    let mut body = String::new();
    decoder.read_to_string(&mut body).unwrap();
    let document = Html::parse_document(&body);

    // let mut file = File::create("test.html").map_err(Internal.from_srcf())?;
    // file.write_all(body.as_bytes())
    //     .map_err(Internal.from_srcf())?;

    let selector = Selector::parse(".normal_reserve_item_name").unwrap();
    let title = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("タイトルが見つかりませんでした"))?
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();

    let mut image_urls: Vec<url::Url> = vec![];
    let selector = Selector::parse(".sale_desc img").unwrap();
    for element in document.select(&selector) {
        let mut image_url = element
            .value()
            .attr("src")
            .ok_or(Internal.with("画像URLが見つかりませんでした"))?
            .to_string();
        if image_url.starts_with("//") {
            image_url = format!("https:{}", image_url);
        }
        image_urls.push(url::Url::parse(image_url.as_str()).map_err(Internal.from_srcf())?);
    }

    let selector = Selector::parse("#priceCalculationConfig").unwrap();
    let actual_price = document
        .select(&selector)
        .next()
        .ok_or(Internal.with("値段が見つかりませんでした"))?
        .value()
        .attr("data-price")
        .ok_or(Internal.with("値段が見つかりませんでした"))?
        .to_string();

    let mut breadcrumb: Vec<String> = vec![];
    let selector = Selector::parse(".sdtext a").unwrap();
    for element in document.select(&selector) {
        let v = element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string();
        breadcrumb.push(v);
    }

    Ok(product.update(
        Some(title),
        image_urls,
        None,
        Some(actual_price),
        None,
        breadcrumb,
        time::now(),
    ))
}
