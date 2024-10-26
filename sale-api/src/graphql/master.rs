use crate::graphql::errors;
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::HttpRequest;
use async_graphql::{Context, EmptySubscription, Object};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_trait::async_trait;
use sale::errors::Kind::{BadRequest, Unauthorized};
use sale::infra::aws::ddb;
use sale::{di, AppResult};

#[derive(Debug, Clone)]
pub struct Authorized {}

#[async_trait]
trait AppContext {
    fn verified(&self) -> Result<Authorized, errors::Error>;
}
#[async_trait]
impl<'a> AppContext for Context<'_> {
    fn verified(&self) -> Result<Authorized, errors::Error> {
        match self.data::<AppResult<Authorized>>()? {
            Ok(v) => Ok(v.clone()),
            Err(err) => Err(match err.kind {
                _ => Unauthorized
                    .with(format!("authorization error: {}", err))
                    .into(),
            }),
        }
    }
}

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

#[derive(Clone)]
pub struct HttpHandler {
    pub schema: Schema,
    pub master_api_token: String,
}

impl HttpHandler {
    pub async fn new() -> Self {
        let envs = di::ENVIRONMENTS.clone();

        let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
            .data(di::DB_PRODUCT_REPOSITORY.get().await.clone())
            .finish();

        HttpHandler {
            schema,
            master_api_token: envs.master_api_token,
        }
    }

    pub async fn handle(&self, http_req: HttpRequest, gql_req: GraphQLRequest) -> GraphQLResponse {
        let mut gql_req = gql_req.into_inner();
        let headers: HeaderMap = HeaderMap::from_iter(http_req.headers().clone().into_iter());

        gql_req = gql_req.data(match headers.get("authorization") {
            None => Err(Unauthorized.into()),
            Some(hv) => verify_token(hv, &self.master_api_token).await,
        });

        self.schema.execute(gql_req).await.into()
    }
}

async fn verify_token(hv: &HeaderValue, master_api_token: &String) -> AppResult<Authorized> {
    let token_str = hv
        .to_str()
        .map_err(BadRequest.from_srcf())?
        .strip_prefix("Bearer ")
        .ok_or_else(|| BadRequest.with("invalid authorization header"))?;

    if token_str.to_string() != master_api_token.to_string() {
        return Err(BadRequest.with("invalid token"));
    }

    Ok(Authorized {})
}

#[derive(Default)]
pub struct Query;
#[Object]
impl Query {
    async fn hello(&self, ctx: &Context<'_>) -> Result<bool, errors::Error> {
        ctx.verified()?;
        Ok(true)
    }
}

#[derive(Default)]
pub struct Mutation;
#[Object]
impl Mutation {
    async fn migrate(&self, ctx: &Context<'_>) -> Result<bool, errors::Error> {
        ctx.verified()?;
        let product_repo = ctx.data::<ddb::types::product::Repository>()?;

        let products = product_repo.find_all().await?;
        for product in products {
            product_repo.put(product.clone()).await?;
        }

        Ok(true)
    }
}
