use actix_web::web::Data;
use actix_web::{self};
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};

use async_graphql::http::GraphiQLSource;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use sea_orm::DatabaseConnection;

use std::env;

use migration::{Migrator, MigratorTrait};

#[cfg(debug_assertions)]
use dotenv::dotenv;

mod api;
mod authorization;
mod ingredients;
mod recipes;
mod tags;
mod token;
mod users;

async fn index(
    schema: web::Data<api::RecipesSchema>,
    req: GraphQLRequest,
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> GraphQLResponse {
    if let Some(hdr) = http_req.headers().get("Authorization") {
        if let Ok(auth) = hdr.to_str() {
            if auth.starts_with("Bearer ") {
                let token = auth[7..].to_string();
                if let Ok(user) = token::decode_jwt(&token, &db).await {
                    return schema.execute(req.into_inner().data(user)).await.into();
                }
            }
        }
    }

    schema.execute(req.into_inner()).await.into()
}

async fn index_graphiql() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();

    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let addrs = env::var("LISTEN").expect("HOST is not set");

    let conn = sea_orm::Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    Migrator::up(&conn, None).await.expect("migration failed");

    let schema = api::create_schema(conn.clone());

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .app_data(Data::new(schema.clone()))
            .app_data(Data::new(conn.clone()))
            .wrap(cors)
            .service(web::resource("/graphql").guard(guard::Post()).to(index))
            .service(web::resource("/graphql").guard(guard::Get()).to(index_graphiql))
    })
    .bind(addrs)?
    .run()
    .await
}
