use crate::graphql::data_loader;
use crate::graphql::service::mutation::MutationRoot;
use crate::graphql::service::query::QueryRoot;
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::HttpRequest;
use async_graphql::EmptySubscription;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use sale::errors::AppError;
use sale::errors::Kind::Unauthorized;
use sale::{di, domain, AppResult};

mod mutation;
mod query;
mod types;

type AuthorizedUserId = domain::user::Id;

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Clone)]
pub struct HttpHandler {
    schema: Schema,
    is_prod: bool,
}

impl HttpHandler {
    pub async fn new() -> Self {
        let envs = di::ENVIRONMENTS.clone();

        let schema = Schema::build(
            QueryRoot::default(),
            MutationRoot::default(),
            EmptySubscription,
        )
        .data(di::DB_PRODUCT_REPOSITORY.get().await.clone())
        .data(data_loader::ProductLoader::from(
            di::DB_PRODUCT_REPOSITORY.get().await.clone(),
        ))
        .finish();

        HttpHandler {
            schema,
            is_prod: envs.is_prod(),
        }
    }

    pub async fn handle(&self, http_req: HttpRequest, gql_req: GraphQLRequest) -> GraphQLResponse {
        let mut gql_req = gql_req.into_inner();

        let headers: HeaderMap = HeaderMap::from_iter(http_req.headers().clone().into_iter());
        gql_req = gql_req.data(match headers.get("authorization") {
            None => Err(Unauthorized.into()),
            Some(hv) => self.verify_token(hv).await,
        });

        if !self.is_prod {
            if let Some(hv) = headers.get("x-debug-user-id") {
                if let Some(v) = hv.to_str().ok() {
                    gql_req = gql_req.data(Ok::<AuthorizedUserId, AppError>(v.to_string().into()));
                }
            }
        }

        self.schema.execute(gql_req).await.into()
    }

    async fn verify_token(&self, hv: &HeaderValue) -> AppResult<AuthorizedUserId> {
        Ok(domain::user::Id::generate())
    }
}
