use async_graphql::*;

use sea_orm::DatabaseConnection;

use crate::{
    authorization::{authorized, users_policy::UsersPolicy, DefaultActions},
    users::UserInput,
};

#[derive(Default)]
pub struct UsersQueries;

#[derive(Default)]
pub struct UsersMutations;

#[Object]
impl UsersQueries {
    async fn users(
        &self,
        ctx: &Context<'_>,
        limit: u64,
        offset: u64,
        search: Option<String>,
    ) -> Result<Vec<entity::users::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(UsersPolicy, DefaultActions::List, user, None, db)?;

        crate::users::list_users(limit, offset, search, db)
            .await
            .map_err(|e| e.into())
    }

    async fn count_users(&self, ctx: &Context<'_>, search: Option<String>) -> Result<usize> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(UsersPolicy, DefaultActions::List, user, None, db)?;

        crate::users::count_users(search, db).await.map_err(|e| e.into())
    }

    async fn user(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::users::Model>> {
        let current_user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let user = crate::users::get_user(id, db).await?;
        authorized(UsersPolicy, DefaultActions::Get, current_user, user.as_ref(), db)?;

        Ok(user)
    }
}

#[Object]
impl UsersMutations {
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> Result<entity::users::Model> {
        let current_user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(UsersPolicy, DefaultActions::Create, current_user, None, db)?;

        crate::users::create_user(user, db).await.map_err(|e| e.into())
    }

    async fn update_user(&self, ctx: &Context<'_>, id: i64, values: UserInput) -> Result<entity::users::Model> {
        let current_user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let user = crate::users::get_user(id, db).await?;
        authorized(UsersPolicy, DefaultActions::Update, current_user, user.as_ref(), db)?;

        crate::users::update_user(id, values, db).await.map_err(|e| e.into())
    }

    async fn delete_user(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let current_user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let user = crate::users::get_user(id, db).await?;
        authorized(UsersPolicy, DefaultActions::Delete, current_user, user.as_ref(), db)?;

        crate::users::delete_user(id, db).await.map_err(|e| e.into())
    }
}
