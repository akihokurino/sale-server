use reqwest::Client;
use sale::domain::time;
use sale::errors::Kind::Internal;
use sale::errors::NotFoundToNone;
use sale::{di, domain, AppResult};
use scraper::{Html, Selector};
use tokio::time::{sleep, Duration};

pub async fn crawl(url: url::Url) -> AppResult<()> {
    let product_repo = di::DB_PRODUCT_REPOSITORY.get().await.clone();

    let products = collect(&url).await?;
    for product in products {
        if let Some(_) = product_repo.get(&product.id).await.not_found_to_none()? {
            println!("すでに存在する商品: {}", product.id.as_str());
            continue;
        }
        product_repo.put(product.clone()).await?;
    }

    let page = url
        .query_pairs()
        .find(|(key, _)| key == "p")
        .map(|(_, value)| value.to_string())
        .ok_or(Internal.with("pが見つかりませんでした"))?;
    let page = page.parse::<u32>().map_err(Internal.from_srcf())?;

    println!("ページ: {}", page);

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

async fn collect(url: &url::Url) -> AppResult<Vec<domain::product::Product>> {
    println!("商品一覧URL: {}", url.as_str());
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(Internal.from_srcf())?;
    let response = client
        .get(url.as_str())
        .send()
        .await
        .map_err(Internal.from_srcf())?;
    if response.status().is_redirection() {
        return Err(Internal.with("リダイレクトが検知されました").into());
    }
    let body = response.text().await.map_err(Internal.from_srcf())?;
    let document = Html::parse_document(&body);

    // let mut file = File::create("test.html").map_err(Internal.from_srcf())?;
    // file.write_all(body.as_bytes())
    //     .map_err(Internal.from_srcf())?;

    let item_selector = Selector::parse(".searchresultitems .searchresultitem").unwrap();
    let url_selector = Selector::parse(".image-link-wrapper--3P6dv").unwrap();
    let points_selector = Selector::parse(".points--AHzKn span").unwrap();

    let mut products: Vec<domain::product::Product> = vec![];
    for element in document.select(&item_selector) {
        let item_id = element
            .value()
            .attr("data-id")
            .ok_or(Internal.with("data-idが見つかりませんでした"))?
            .to_string();

        let shop_id = element
            .value()
            .attr("data-shop-id")
            .ok_or(Internal.with("data-shop-idが見つかりませんでした"))?
            .to_string();

        let e_ref = element
            .select(&url_selector)
            .next()
            .ok_or(Internal.with("URLが見つかりませんでした"))?;
        let url = e_ref
            .value()
            .attr("href")
            .ok_or(Internal.with("URLが見つかりませんでした"))?
            .to_string();
        let url = url::Url::parse(&url).map_err(Internal.from_srcf())?;

        let e_ref = element
            .select(&points_selector)
            .next()
            .ok_or(Internal.with("ポイントが見つかりませんでした"))?;
        let points = e_ref.text().collect::<Vec<_>>().concat().trim().to_string();

        let source = domain::product::Source::Rakuten;
        products.push(domain::product::Product::new(
            domain::product::Id::new(format!("{}-{}-{}", source, shop_id, item_id)),
            source,
            url,
            Some(points),
            time::now(),
        ));
    }

    Ok(products)
}
