use std::env;

use actix_web::web::Data;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::http::GraphiQLSource;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use cookie::{CookieJar, Key};
#[cfg(debug_assertions)]
use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;

mod api;
mod authorization;
mod ingredients;
mod jar;
mod recipes;
mod steps;
mod tags;
mod users;
mod utils;

async fn index(
    schema: web::Data<api::RecipesSchema>,
    req: GraphQLRequest,
    http_req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> GraphQLResponse {
    if let Some(token) = http_req.cookie("recipes_auth") {
        let master_key = env::var("COOKIE_KEY").expect("env variable COOKIE_KEY not set");
        let key = Key::derive_from(master_key.as_bytes());
        let jar = CookieJar::new();
        let priv_jar = jar.private(&key);

        if let Some(value) = priv_jar.decrypt(token) {
            if let Some(user) = users::get_user_by_id(value.value().parse().unwrap(), &db).await {
                return schema.execute(req.into_inner().data(user)).await.into();
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

    log::info!("Listening on http://{}", addrs);

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        let static_path = utils::image_base_path();

        App::new()
            .app_data(Data::new(schema.clone()))
            .app_data(Data::new(conn.clone()))
            .wrap(cors)
            .service(web::resource("/graphql").guard(guard::Post()).to(index))
            .service(web::resource("/graphql").guard(guard::Get()).to(index_graphiql))
            .service(
                actix_files::Files::new("/pictures", static_path)
                    .show_files_listing()
                    .use_last_modified(true),
            )
    })
    .bind(addrs)?
    .run()
    .await
}
