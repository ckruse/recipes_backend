use async_graphql::*;
use sea_orm::DatabaseConnection;

use crate::authorization::{authorized, ingredients_policy::IngredientsPolicy, DefaultActions};
use crate::ingredients::IngredientInput;

#[derive(Default)]
pub struct IngredientsQueries;

#[derive(Default)]
pub struct IngredientsMutations;

#[Object]
impl IngredientsQueries {
    async fn ingredients(
        &self,
        ctx: &Context<'_>,
        limit: u64,
        offset: u64,
        search: Option<String>,
    ) -> Result<Vec<entity::ingredients::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(IngredientsPolicy, DefaultActions::List, user, None, db)?;

        crate::ingredients::list_ingredients(limit, offset, search, db).await
    }

    pub async fn count_ingredients(&self, ctx: &Context<'_>, search: Option<String>) -> Result<u64> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(IngredientsPolicy, DefaultActions::List, user, None, db)?;

        crate::ingredients::count_ingredients(search, db).await
    }

    async fn ingredient(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::ingredients::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let ingredient = crate::ingredients::get_ingredient_by_id(id, db).await?;
        authorized(IngredientsPolicy, DefaultActions::Get, user, ingredient.as_ref(), db)?;

        Ok(ingredient)
    }
}

#[Object]
impl IngredientsMutations {
    async fn create_ingredient(
        &self,
        ctx: &Context<'_>,
        ingredient: IngredientInput,
    ) -> Result<entity::ingredients::Model> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(IngredientsPolicy, DefaultActions::Create, user, None, db)?;
        crate::ingredients::create_ingredient(ingredient, db).await
    }

    async fn update_ingredient(
        &self,
        ctx: &Context<'_>,
        id: i64,
        ingredient: IngredientInput,
    ) -> Result<entity::ingredients::Model> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let existing_ingredient = crate::ingredients::get_ingredient_by_id(id, db).await?;
        authorized(
            IngredientsPolicy,
            DefaultActions::Update,
            user,
            existing_ingredient.as_ref(),
            db,
        )?;

        crate::ingredients::update_ingredient(id, ingredient, db)
            .await
            .map_err(|e| e.into())
    }

    async fn delete_ingredient(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(IngredientsPolicy, DefaultActions::Delete, user, None, db)?;

        crate::ingredients::delete_ingredient(id, db).await
    }
}
