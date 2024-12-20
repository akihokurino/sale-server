DOCKER_CMD_BASE :=
DOCKER_EXTRA_PARAMS :=
ifeq ($(USE_DOCKER), 1)
	DOCKER_CACHE_PARAMS :=
	ifeq ($(USE_DOCKER_CACHE), 1)
		DOCKER_CACHE_PARAMS := -v "$(shell pwd)/.docker/cache/cargo/registry:/root/.cargo/registry"
	endif
	DOCKER_CMD_BASE := docker run --rm -v "$(shell pwd):/volume" $(DOCKER_CACHE_PARAMS) $(DOCKER_EXTRA_PARAMS) clux/muslrust:1.81.0-stable
endif
BIN_OUTPUT_DIR := target/x86_64-unknown-linux-musl/release
SRC_FILES := $(shell find . -type f | grep -v '^\./target' | grep -v '/\.')
DEPLOY_CRATES := sale-api crawler-rakuten scheduled-crawler
$(BIN_OUTPUT_DIR)/%: $(SRC_FILES)
	$(DOCKER_CMD_BASE) cargo build --release --bin $(lastword $(subst /, ,$@)) --target x86_64-unknown-linux-musl
	if [ "$(STRIP)" = "1" ]; then strip $@; fi
CRAWLER_RAKUTEN_LAMBDA_NAME := dev-sale-server-CrawlerRakutenFunction-EBqCiM66oVeo

build-ApiFunction: $(BIN_OUTPUT_DIR)/sale-api
	cp $< $(ARTIFACTS_DIR)/bootstrap

build-CrawlerRakutenFunction: $(BIN_OUTPUT_DIR)/crawler-rakuten
	cp $< $(ARTIFACTS_DIR)/bootstrap

build-ScheduledCrawlerFunction: $(BIN_OUTPUT_DIR)/scheduled-crawler
	cp $< $(ARTIFACTS_DIR)/bootstrap

.PHONY: debug-build
debug-build:
	cargo build

.PHONY: build
build: $(addprefix $(BIN_OUTPUT_DIR)/,$(DEPLOY_CRATES))

.PHONY: crawl-amazon-product-list
crawl-amazon-product-list:
	cd crawler-amazon && npm run crawl_product_list

.PHONY: crawl-amazon-product-detail
crawl-amazon-product-detail:
	cd crawler-amazon && npm run crawl_product_detail

.PHONY: run-local
run-local:
	SSM_DOTENV_PARAMETER_NAME=/sale/dev/server/dotenv WITH_LAMBDA=false cargo run --bin sale-api

.PHONY: run-dev-crawl-rakuten-entrypoint
run-dev-crawl-rakuten-entrypoint:
	aws lambda invoke \
      	--function-name $(CRAWLER_RAKUTEN_LAMBDA_NAME) \
    	--payload '{"body": "CrawlEntrypoint"}' \
    	--cli-binary-format raw-in-base64-out \
    	--cli-read-timeout 0 \
    	/dev/null

.PHONY: run-local-crawl-rakuten-product-list
run-local-crawl-rakuten-product-list:
	SSM_DOTENV_PARAMETER_NAME=/sale/dev/server/dotenv WITH_LAMBDA=false cargo run --bin crawler-rakuten -- '{"body": {"CrawlList": {"url": "https://search.rakuten.co.jp/search/mall/-/551177/?f=13&p=1"}}}'

.PHONY: run-dev-crawl-rakuten-product-list
run-dev-crawl-rakuten-product-list:
	aws lambda invoke \
      	--function-name $(CRAWLER_RAKUTEN_LAMBDA_NAME) \
    	--payload '{"body": {"CrawlList": {"url": "https://search.rakuten.co.jp/search/mall/-/551177/?f=13&p=1"}}}' \
    	--cli-binary-format raw-in-base64-out \
    	--cli-read-timeout 0 \
    	/dev/null

.PHONY: run-local-crawl-rakuten-product-detail
run-local-crawl-rakuten-product-detail:
	SSM_DOTENV_PARAMETER_NAME=/sale/dev/server/dotenv WITH_LAMBDA=false cargo run --bin crawler-rakuten -- '{"body": {"CrawlDetail": {"only_preparing": false}}}'

.PHONY: run-dev-crawl-rakuten-product-detail
run-dev-crawl-rakuten-product-detail:
	aws lambda invoke \
      	--function-name $(CRAWLER_RAKUTEN_LAMBDA_NAME) \
    	--payload '{"body": {"CrawlDetail": {"only_preparing": false}}}' \
    	--cli-binary-format raw-in-base64-out \
    	--cli-read-timeout 0 \
    	/dev/null

.PHONY: deploy
deploy: $(addprefix $(BIN_OUTPUT_DIR)/,$(DEPLOY_CRATES))
	sam build
	sam deploy --config-file $(SAM_CONFIG_FILE) --no-confirm-changeset --no-fail-on-empty-changeset

.PHONY: deploy-dev
deploy-dev:
	make SAM_CONFIG_FILE=dev.samconfig.toml deploy
