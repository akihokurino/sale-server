.PHONY: debug-build
debug-build:
	cargo build

.PHONY: crawl_amazon_product_list
crawl_amazon_product_list:
	cd crawler-amazon && npm run crawl_product_list

.PHONY: crawl_amazon_product_detail
crawl_amazon_product_detail:
	cd crawler-amazon && npm run crawl_product_detail

.PHONY: crawl_rakuten_product_list
crawl_rakuten_product_list:
	cargo run --bin crawler-rakuten

.PHONY: crawl_rakuten_product_detail
crawl_rakuten_product_detail:
	cargo run --bin crawler-rakuten

.PHONY: run-local
run-local:
	SSM_DOTENV_PARAMETER_NAME=/sale/dev/server/dotenv cargo run --bin sale-api