use async_graphql::*;
use sea_orm::DatabaseConnection;

use crate::{token::create_jwt, users::authenticate_user};

#[derive(Default)]
pub struct SessionMutations;

#[derive(SimpleObject)]
struct LoginPayload {
    token: String,
    user: entity::users::Model,
}

#[Object]
impl SessionMutations {
    async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<LoginPayload> {
        let db = ctx.data::<DatabaseConnection>()?;

        if let Some((token, user)) = authenticate_user(email, password, db).await {
            Ok(LoginPayload { token, user })
        } else {
            Err("Invalid credentials".into())
        }
    }

    async fn refresh(&self, ctx: &Context<'_>) -> Result<LoginPayload> {
        if let Some(user) = ctx.data_opt::<entity::users::Model>() {
            Ok(LoginPayload {
                token: create_jwt(&user)?,
                user: user.clone(),
            })
        } else {
            Err("Invalid credentials".into())
        }
    }
}
