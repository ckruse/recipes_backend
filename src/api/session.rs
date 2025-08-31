use ::http::header::SET_COOKIE;
use async_graphql::*;
use jwt_simple::prelude::*;

use crate::AppState;
use crate::users::authenticate_user;

#[derive(Default)]
pub struct SessionMutations;

#[derive(Clone, Debug, SimpleObject)]
struct AuthResponse {
    user: entity::users::Model,
    token: String,
}

#[Object]
impl SessionMutations {
    async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthResponse> {
        let state = ctx.data::<AppState>()?;

        let Some(user) = authenticate_user(email, password, &state.conn).await else {
            return Err("Invalid credentials".into());
        };

        let claims = Claims::create(Duration::from_days(30))
            .with_issuer("Recipes")
            .with_subject(user.id.to_string());

        let token = state.token_key.authenticate(claims)?;

        #[cfg(not(debug_assertions))]
        ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly; Secure"));

        #[cfg(debug_assertions)]
        ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly"));

        Ok(AuthResponse { user, token })
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<AuthResponse> {
        let state = ctx.data::<AppState>()?;

        let Some(user) = ctx.data::<Option<entity::users::Model>>()? else {
            return Err("Invalid credentials".into());
        };

        let claims = Claims::create(Duration::from_days(30))
            .with_issuer("Recipes")
            .with_subject(user.id.to_string());
        let token = state.token_key.authenticate(claims)?;

        #[cfg(not(debug_assertions))]
        ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly; Secure"));

        #[cfg(debug_assertions)]
        ctx.append_http_header(SET_COOKIE, format!("recipes_auth={token}; Path=/; HttpOnly"));

        Ok(AuthResponse {
            user: user.clone(),
            token,
        })
    }
}
