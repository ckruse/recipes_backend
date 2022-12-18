use async_graphql::*;
use sea_orm::DatabaseConnection;

use crate::authorization::{authorized, recipes_policy::RecipesPolicy, DefaultActions};
use crate::recipes::RecipeInput;

#[derive(Default)]
pub struct RecipesQueries;

#[derive(Default)]
pub struct RecipesMutations;

#[Object]
impl RecipesQueries {
    async fn recipes(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(max_length = 255))] search: Option<String>,
        #[graphql(validator(max_items = 3))] tags: Option<Vec<String>>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<entity::recipes::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(RecipesPolicy, DefaultActions::List, user, None, db)?;

        let search = search.map(|s| s.split_whitespace().map(|s| s.to_lowercase()).collect());

        crate::recipes::list_recipes(limit, offset, search, tags, db)
            .await
            .map_err(|e| e.into())
    }

    pub async fn count_recipes(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<u64> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(RecipesPolicy, DefaultActions::List, user, None, db)?;

        let search = search.map(|s| s.split_whitespace().map(|s| s.to_lowercase()).collect());

        crate::recipes::count_recipes(search, tags, db)
            .await
            .map_err(|e| e.into())
    }

    async fn recipe(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::recipes::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = crate::recipes::get_recipe_by_id(id, db).await?;

        authorized(RecipesPolicy, DefaultActions::Get, user, recipe.as_ref(), db)?;

        Ok(recipe)
    }

    async fn random_recipe(&self, ctx: &Context<'_>) -> Result<Option<entity::recipes::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = crate::recipes::get_random_recipe(db).await?;

        authorized(RecipesPolicy, DefaultActions::Get, user, recipe.as_ref(), db)?;

        Ok(recipe)
    }

    async fn random_recipes(&self, ctx: &Context<'_>, limit: u64) -> Result<Vec<entity::recipes::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(RecipesPolicy, DefaultActions::List, user, None, db)?;

        crate::recipes::get_random_recipes(limit, db)
            .await
            .map_err(|e| e.into())
    }
}

#[Object]
impl RecipesMutations {
    async fn update_recipe(&self, ctx: &Context<'_>, id: i64, recipe: RecipeInput) -> Result<entity::recipes::Model> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let file = recipe.image.as_ref().map(|picture| picture.value(ctx).unwrap());

        let existing_recipe = crate::recipes::get_recipe_by_id(id, db).await?;

        authorized(
            RecipesPolicy,
            DefaultActions::Update,
            user,
            existing_recipe.as_ref(),
            db,
        )?;

        crate::recipes::update_recipe(id, recipe, file, db)
            .await
            .map_err(|e| e.into())
    }

    async fn create_recipe(&self, ctx: &Context<'_>, recipe: RecipeInput) -> Result<entity::recipes::Model> {
        let user = ctx.data::<entity::users::Model>()?;
        let db = ctx.data::<DatabaseConnection>()?;

        let file = recipe.image.as_ref().map(|picture| picture.value(ctx).unwrap());

        authorized(RecipesPolicy, DefaultActions::Create, Some(user), None, db)?;

        crate::recipes::create_recipe(recipe, file, user.id, db)
            .await
            .map_err(|e| e.into())
    }

    async fn delete_recipe(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = crate::recipes::get_recipe_by_id(id, db).await?;

        authorized(RecipesPolicy, DefaultActions::Delete, user, recipe.as_ref(), db)?;

        crate::recipes::delete_recipe(id, db).await.map_err(|e| e.into())
    }
}
