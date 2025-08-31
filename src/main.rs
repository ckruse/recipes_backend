use std::env;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Extension, Router};
use entity::users::Model as User;
use jwt_simple::prelude::*;
use migration::{Migrator, MigratorTrait};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::types::AppState;

mod api;
mod authorization;
mod bring;
mod current_user;
mod ingredients;
mod recipes;
mod steps;
mod tags;
mod types;
mod users;
mod utils;
mod weekplan;

async fn index(
    Extension(schema): Extension<api::RecipesSchema>,
    Extension(user): Extension<Option<User>>,
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner().data(state).data(user)).await.into()
}

async fn index_graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let addrs = env::var("LISTEN").expect("LISTEN is not set");
    let token_key = HS512Key::from_bytes(std::env::var("JWT_KEY").expect("JWT_KEY not set").as_bytes());

    let conn = sea_orm::Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    Migrator::up(&conn, None).await.expect("migration failed");

    log::info!("🚀 Listening on http://{}", addrs);

    let schema = api::create_schema(conn.clone());
    let state = AppState { conn, token_key };
    let pictures_static_path = utils::image_base_path();
    let avatars_static_path = utils::avatar_base_path();

    let listener = TcpListener::bind(addrs).await.expect("could not create listener");

    let mut router = Router::new()
        .nest_service("/pictures", ServeDir::new(pictures_static_path))
        .nest_service("/avatars", ServeDir::new(avatars_static_path))
        .route("/graphql", get(index_graphiql).post(index))
        .merge(bring::routes());

    router = router
        .layer(Extension(schema))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), current_user::current_user));

    #[cfg(debug_assertions)]
    {
        use http::{Method, header};
        use tower_http::cors::CorsLayer;

        router = router.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::COOKIE, header::CONTENT_TYPE])
                .allow_origin(["http://localhost:3000".parse::<http::HeaderValue>().unwrap()])
                .allow_credentials(true),
        );
    }

    axum::serve(listener, router.with_state(state).into_make_service())
        .await
        .unwrap();
}
