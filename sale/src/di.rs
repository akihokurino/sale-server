use crate::infra::aws::{ddb, lambda, sns, ssm};
use crate::sync::LazyAsync;
use crate::{env, lazy_async};
use aws_config::BehaviorVersion;
use once_cell::sync::Lazy;

static SSM_PARAMETER_NAME: Lazy<String> = Lazy::new(|| {
    std::env::var("SSM_DOTENV_PARAMETER_NAME").expect("SSM_DOTENV_PARAMETER_NAME should set")
});
pub static ENVIRONMENTS: Lazy<env::Environments> = Lazy::new(|| env::Environments::new());

static AWS_CONFIG: LazyAsync<aws_types::SdkConfig> =
    lazy_async!(aws_config::defaults(BehaviorVersion::latest()).load());
static SSM_CLIENT: LazyAsync<aws_sdk_ssm::Client> =
    lazy_async!(async { aws_sdk_ssm::Client::new(AWS_CONFIG.get().await) });
static DDB_CLIENT: LazyAsync<aws_sdk_dynamodb::Client> =
    lazy_async!(async { aws_sdk_dynamodb::Client::new(AWS_CONFIG.get().await) });
static LAMBDA_CLIENT: LazyAsync<aws_sdk_lambda::Client> = lazy_async!(async {
    let ses_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    aws_sdk_lambda::Client::new(&ses_config)
});
static SNS_CLIENT: LazyAsync<aws_sdk_sns::Client> = lazy_async!(async {
    let ses_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    aws_sdk_sns::Client::new(&ses_config)
});

pub static SSM_ADAPTER: LazyAsync<ssm::Adapter> = lazy_async!(async {
    ssm::Adapter::new(SSM_CLIENT.get().await.clone(), SSM_PARAMETER_NAME.clone())
});
pub static LAMBDA_ADAPTER: LazyAsync<lambda::Adapter> =
    lazy_async!(async { lambda::Adapter::new(LAMBDA_CLIENT.get().await.clone(),) });
pub static SNS_ADAPTER: LazyAsync<sns::Adapter> =
    lazy_async!(async { sns::Adapter::new(SNS_CLIENT.get().await.clone(),) });

async fn ddb_repo<E>() -> ddb::TableRepository<E> {
    ddb::TableRepository::new(
        DDB_CLIENT.get().await.clone(),
        DDB_TABLE_NAME_PROVIDER.clone(),
    )
}
static DDB_TABLE_NAME_PROVIDER: Lazy<ddb::TableNameProvider> =
    Lazy::new(|| ddb::TableNameProvider::new(format!("{}-sale-", ENVIRONMENTS.clone().env)));
pub static DB_PRODUCT_REPOSITORY: LazyAsync<ddb::types::product::Repository> =
    lazy_async!(ddb_repo());
