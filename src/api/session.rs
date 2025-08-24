use async_graphql::*;
use sea_orm::DatabaseConnection;

use crate::jar::get_auth_cookie;
use crate::users::authenticate_user;

#[derive(Default)]
pub struct SessionMutations;

#[Object]
impl SessionMutations {
    async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<entity::users::Model> {
        let db = ctx.data::<DatabaseConnection>()?;

        if let Some(user) = authenticate_user(email, password, db).await {
            let cookie = get_auth_cookie(&user);
            ctx.insert_http_header(::http::header::SET_COOKIE, cookie);

            Ok(user)
        } else {
            Err("Invalid credentials".into())
        }
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<entity::users::Model> {
        if let Some(user) = ctx.data_opt::<entity::users::Model>() {
            let cookie = get_auth_cookie(user);
            ctx.insert_http_header(::http::header::SET_COOKIE, cookie);

            Ok(user.clone())
        } else {
            Err("Invalid credentials".into())
        }
    }
}
