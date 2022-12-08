use async_graphql::*;
use sea_orm::DatabaseConnection;

use crate::authorization::recipes_policy::RecipesPolicy;
use crate::authorization::{authorized, DefaultActions};
use crate::recipes;
use crate::steps::StepInput;

#[derive(Default)]
pub struct StepsQueries;

#[derive(Default)]
pub struct StepsMutations;

#[Object]
impl StepsQueries {
    async fn steps(&self, ctx: &Context<'_>, recipe_id: i64) -> Result<Vec<entity::steps::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = recipes::get_recipe_by_id(recipe_id, db).await?;
        authorized(RecipesPolicy, DefaultActions::Get, user, recipe.as_ref(), db)?;

        crate::steps::list_steps(recipe_id, db).await.map_err(|e| e.into())
    }

    async fn count_steps(&self, ctx: &Context<'_>, recipe_id: i64) -> Result<u64> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = recipes::get_recipe_by_id(recipe_id, db).await?;
        authorized(RecipesPolicy, DefaultActions::Get, user, recipe.as_ref(), db)?;

        crate::steps::count_steps(recipe_id, db).await.map_err(|e| e.into())
    }

    async fn step(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::steps::Model>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let step = crate::steps::get_step_by_id(id, db).await?;

        if let Some(step) = step {
            let recipe = recipes::get_recipe_by_id(step.recipe_id, db).await?;
            authorized(RecipesPolicy, DefaultActions::Get, user, recipe.as_ref(), db)?;

            return Ok(Some(step));
        }

        Err(ServerError::new("Step not found", Some(ctx.item.pos)).into())
    }
}

#[Object]
impl StepsMutations {
    async fn create_step(&self, ctx: &Context<'_>, recipe_id: i64, step: StepInput) -> Result<entity::steps::Model> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let recipe = recipes::get_recipe_by_id(recipe_id, db).await?;
        authorized(RecipesPolicy, DefaultActions::Update, user, recipe.as_ref(), db)?;

        crate::steps::create_step(recipe_id, &step, db)
            .await
            .map_err(|e| e.into())
    }

    async fn update_step(&self, ctx: &Context<'_>, id: i64, step: StepInput) -> Result<entity::steps::Model> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let existing_step = crate::steps::get_step_by_id(id, db)
            .await?
            .ok_or_else(|| ServerError::new("Step not found", Some(ctx.item.pos)))?;
        let recipe = recipes::get_recipe_by_id(existing_step.recipe_id, db).await?;
        authorized(RecipesPolicy, DefaultActions::Update, user, recipe.as_ref(), db)?;

        crate::steps::update_step(id, step, db).await.map_err(|e| e.into())
    }

    async fn delete_step(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let step = crate::steps::get_step_by_id(id, db)
            .await?
            .ok_or_else(|| ServerError::new("Step not found", Some(ctx.item.pos)))?;

        let recipe = recipes::get_recipe_by_id(step.recipe_id, db).await?;
        authorized(RecipesPolicy, DefaultActions::Update, user, recipe.as_ref(), db)?;

        crate::steps::delete_step(step, db).await.map_err(|e| e.into())
    }
}
