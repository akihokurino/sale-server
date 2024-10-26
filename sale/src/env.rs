use std::str::FromStr;

fn must_env(k: &str) -> String {
    std::env::var(k).expect(format!("env {} missing", k).as_str())
}

#[derive(Debug, Clone)]
pub struct Environments {
    pub env: String,
    pub port: String,
    pub with_lambda: bool,
    pub master_api_token: String,
    pub crawler_rakuten_lambda_arn: String,
    pub crawler_rakuten_sns_arn: String,
}
impl Environments {
    pub fn new() -> Self {
        Environments {
            env: must_env("ENV"),
            port: std::env::var("PORT").unwrap_or("4000".to_string()),
            master_api_token: must_env("MASTER_API_TOKEN"),
            with_lambda: std::env::var("WITH_LAMBDA")
                .map(|v| bool::from_str(&v).expect("failed to parse WITH_LAMBDA"))
                .unwrap_or(true),
            crawler_rakuten_lambda_arn: must_env("CRAWLER_RAKUTEN_LAMBDA_ARN"),
            crawler_rakuten_sns_arn: must_env("CRAWLER_RAKUTEN_SNS_ARN"),
        }
    }

    pub fn is_prod(&self) -> bool {
        self.env == "prod"
    }
}
