import puppeteer from "puppeteer";

const setTimeout = require("node:timers/promises").setTimeout;

const run = async () => {
    const browser = await puppeteer.launch({
        headless: false,
        args: [
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "-–disable-dev-shm-usage",
            "--disable-gpu",
            "--no-first-run",
            "--no-zygote",
            "--single-process",
            "--start-maximized",
        ],
    });

    const page = await browser.newPage();
    await page.setViewport({width: 1920, height: 1080});
    await page.goto("https://www.amazon.co.jp/dp/B0CJX4M5WB/ref=syn_sd_onsite_desktop_0?ie=UTF8&pd_rd_plhdr=t&aref=kcIsXTwzZ1&th=1");

    const title = await page.$eval("#productTitle", el => el?.textContent?.trim() || "");
    console.log("タイトル:", title);

    const imageUrls = await page.$$eval("#altImages ul li img", imgs =>
        imgs.map(img => img.getAttribute("src") || "")
    );
    imageUrls.forEach((imgUrl, index) => console.log(`画像URL ${index + 1}: ${imgUrl}`));

    const retailPrice = await page.$eval(".basisPrice .a-offscreen", el => el?.textContent?.trim() || "");
    console.log("元値:", retailPrice);

    const price = await page.$eval("#corePrice_feature_div .a-offscreen", el => el?.textContent?.trim() || "");
    console.log("値段:", price);

    const discount = await page.$eval(".savingsPercentage", el => el?.textContent?.trim() || "");
    console.log("割引率:", discount);

    const points = await page.$eval("#points_feature_div span", el => el?.textContent?.replace(/\u00a0/g, ' ').trim() || "");
    console.log("ポイント:", points);

    const shipper = await page.$$eval(".offer-display-feature-label", elements => {
        for (let element of elements) {
            const label = element.querySelector("span")?.textContent?.trim();
            if (label === "出荷元") {
                const nextDiv = element.nextElementSibling?.querySelector(".offer-display-feature-text-message");
                return nextDiv?.textContent?.trim() || "";
            }
        }
        throw new Error("出荷元が見つかりません");
    });
    console.log("出荷元:", shipper);

    const seller = await page.$$eval(".offer-display-feature-label", elements => {
        for (let element of elements) {
            const label = element.querySelector("span")?.textContent?.trim();
            if (label === "販売元") {
                const nextDiv = element.nextElementSibling?.querySelector(".offer-display-feature-text-message");
                return nextDiv?.textContent?.trim() || "";
            }
        }
        throw new Error("販売元が見つかりません");
    });
    console.log("販売元:", seller);

    const breadcrumbs = await page.$$eval("#wayfinding-breadcrumbs_feature_div ul li a", links =>
        links.map(link => link?.textContent?.trim() || "")
    );
    breadcrumbs.forEach((breadcrumb, index) => console.log(`パンクズ ${index + 1}: ${breadcrumb}`));

    // 商品の情報までスクロール
    let detailExists = false;
    while (!detailExists) {
        detailExists = await page.evaluate(() => {
            const d = document.querySelector("#productDetails_feature_div");
            if (d) {
                d.scrollIntoView({behavior: "smooth", block: "start"});
                return true;
            }
            return false;
        });

        if (!detailExists) {
            await page.evaluate(() => {
                window.scrollBy(0, 300);
            });
            await setTimeout(500);
        }
    }
    await setTimeout(500);

    // 「商品詳細」というテキストを持つ要素をクリックして展開
    const expandDetails = await page.$$eval('span.a-expander-prompt', elements => {
        const element = elements.find(el => el.textContent?.trim() === '商品詳細');
        if (element) {
            element.click();
            return true;
        }
        return false;
    });

    // たまにHTMLの構造が変わる
    let productDetailDom = expandDetails ? "#productDetails_expanderTables_depthRightSections" : "#productDetails_db_sections"

    const asin = await page.$$eval(`${productDetailDom} tr`, rows => {
        for (let row of rows) {
            const th = row.querySelector("th")?.textContent?.trim();
            console.log(th)
            if (th === "ASIN") {
                return row.querySelector("td")?.textContent?.trim() || "";
            }
        }
        throw new Error("ASINが見つかりません");
    });
    console.log("ASIN:", asin);

    // おすすめ度
    const recommend = await page.$$eval(`${productDetailDom} tr`, rows => {
        for (let row of rows) {
            const th = row.querySelector("th")?.textContent?.trim();
            console.log(th)
            if (th === "おすすめ度") {
                const ratingSpan = row.querySelector("td .a-icon-alt");
                if (ratingSpan) {
                    return ratingSpan.textContent?.trim() || "";
                }
                return "";
            }
        }
        throw new Error("おすすめ度が見つかりません");
    });
    console.log("おすすめ度:", recommend);


    await browser.close();
};

(async () => {
    await run();
})();
