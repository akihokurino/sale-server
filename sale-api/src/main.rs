use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

mod graphql;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_http_handler = graphql::service::HttpHandler::new().await;
    let is_prod = false;
    let with_lambda = false;

    let app_factory = move || {
        let mut app = App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allowed_methods(["GET", "POST"])
                    .max_age(3600)
                    .supports_credentials(),
            )
            .app_data(Data::new(service_http_handler.clone()))
            .service(
                web::resource("/service/graphql")
                    .guard(guard::Post())
                    .to(service_graphql_route),
            );
        if !is_prod {
            app = app.service(
                web::resource("/service/playground")
                    .guard(guard::Get())
                    .to(|| async { handle_playground("service") }),
            );
        }

        app
    };

    if with_lambda {
        lambda_web::run_actix_on_lambda(app_factory)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    } else {
        println!("listen as http server on port {}", 4000);
        HttpServer::new(app_factory)
            .bind(format!("127.0.0.1:{}", 4000))?
            .run()
            .await
    }
}

async fn service_graphql_route(
    handler: Data<graphql::service::HttpHandler>,
    http_req: HttpRequest,
    gql_req: GraphQLRequest,
) -> GraphQLResponse {
    handler.handle(http_req, gql_req).await
}

fn handle_playground(schema_name: &'static str) -> actix_web::Result<HttpResponse> {
    let path = format!("/{}/graphql", schema_name);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new(&path).subscription_endpoint(&path),
        )))
}
