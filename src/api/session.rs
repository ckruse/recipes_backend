use ::http::header::SET_COOKIE;
use async_graphql::*;
use jwt_simple::prelude::*;

use crate::{AppState, users::authenticate_user};

#[derive(Default)]
pub struct SessionMutations;

#[Object]
impl SessionMutations {
    async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<entity::users::Model> {
        let state = ctx.data::<AppState>()?;

        if let Some(user) = authenticate_user(email, password, &state.conn).await {
            let claims = Claims::create(Duration::from_days(30))
                .with_issuer("Recipes")
                .with_subject(user.id.to_string());

            let token = state.token_key.authenticate(claims)?;

            #[cfg(not(debug_assertions))]
            ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly; Secure"));

            #[cfg(debug_assertions)]
            ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly"));

            Ok(user)
        } else {
            Err("Invalid credentials".into())
        }
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<entity::users::Model> {
        let state = ctx.data::<AppState>()?;

        if let Some(user) = ctx.data::<Option<entity::users::Model>>()? {
            let claims = Claims::create(Duration::from_days(30))
                .with_issuer("Recipes")
                .with_subject(user.id.to_string());
            let token = state.token_key.authenticate(claims)?;

            #[cfg(not(debug_assertions))]
            ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly; Secure"));

            #[cfg(debug_assertions)]
            ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly"));

            Ok(user.clone())
        } else {
            Err("Invalid credentials".into())
        }
    }
}
