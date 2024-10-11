.PHONY: debug-build
debug-build:
	cargo build

crawl_amazon_product_list:
	cd crawler-amazon && npm run crawl_product_list

crawl_amazon_product_detail:
	cd crawler-amazon && npm run crawl_product_detail

crawl_rakuten_product_list:
	cargo run --bin crawler-rakuten

crawl_rakuten_product_detail:
	cargo run --bin crawler-rakuten