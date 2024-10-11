import puppeteer from "puppeteer";

const setTimeout = require("node:timers/promises").setTimeout;

// amazonではスクレイピングの対策で、JSで現在必要な（見えている）indexのdivのみが表示される仕組みになっている
// <div data-testid="virtuoso-item-list">
//   <div data-index="0">
//     <a data-testid="product-card-link"></a>
//     <a data-testid="product-card-link"></a>
//   </div>
//   <div data-index="1">
//     <a data-testid="product-card-link"></a>
//     <a data-testid="product-card-link"></a>
//   </div>
// </div>
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
    await page.goto("https://www.amazon.co.jp/gp/goldbox");

    let allLinks = new Set<string>();

    for (let i = 0; ; i++) {
        try {
            // div[data-index="i"] のところまでスクロール
            const rowExists = await page.evaluate((index) => {
                const d = document.querySelector(`div[data-index="${index}"]`);
                if (d) {
                    d.scrollIntoView({behavior: "smooth", block: "end"});
                    return true;
                }
                return false;
            }, i);

            // 少し間を置いてから次のindexのdivが表示されるところまでスクロール
            // 300pxはdiv[data-index="i"]の高さ

            await setTimeout(500);
            await page.evaluate(() => {
                window.scrollBy(0, 300);
            });
            await setTimeout(1000);

            // div[data-index="i"]のdivが存在しない場合はループを終了
            if (!rowExists) {
                console.log(`waiting next button click...`);

                const nextButtonExists = await page.evaluate(() => {
                    const d = document.querySelector(
                        'button[data-testid="load-more-view-more-button"]'
                    );
                    if (d) {
                        d.scrollIntoView({behavior: "smooth", block: "end"});
                        return true;
                    }
                    return false;
                });

                if (nextButtonExists) {
                    // 「さらにセールを表示する」ボタンがある場合、クリックして再度ループを開始
                    await page.click('button[data-testid="load-more-view-more-button"]');
                    await setTimeout(2000);
                } else {
                    // ボタンがない場合はループを終了
                    console.log("No more 'さらにセールを表示する' button, ending loop.");
                    break;
                }
            }

            // div[data-index="i"]のdivの中に存在するリンクを集める
            const currentLinks = await page.evaluate((index) => {
                const div = document.querySelector(`div[data-index="${index}"]`);
                if (div) {
                    const anchors = div.querySelectorAll(
                        'a[data-testid="product-card-link"]'
                    );
                    return Array.from(anchors)
                        .map((anchor) => (anchor as HTMLAnchorElement).href)
                        .filter((href) => href);
                }
                return [];
            }, i);
            currentLinks.forEach((link) => allLinks.add(link));
            console.log(`${i} cell links: ${currentLinks.length}`);
        } catch (e) {
            console.error(`Error processing div[data-index="${i}"]:`, e);
            break;
        }
    }

    const links = Array.from(allLinks);
    console.log(`Total number of links: ${links.length}`);

    await browser.close();
};

(async () => {
    await run();
})();