use async_graphql::*;

use sea_orm::DatabaseConnection;

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
        search: Option<String>,
        tags: Option<Vec<String>>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<entity::recipes::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let search = match search {
            Some(s) => Some(s.split_whitespace().map(|s| s.to_string()).collect()),
            None => None,
        };

        crate::recipes::list_recipes(limit, offset, search, tags, db)
            .await
            .map_err(|e| e.into())
    }

    pub async fn count_recipes(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<usize> {
        let db = ctx.data::<DatabaseConnection>()?;

        let search = match search {
            Some(s) => Some(s.split_whitespace().map(|s| s.to_string()).collect()),
            None => None,
        };

        crate::recipes::count_recipes(search, tags, db)
            .await
            .map_err(|e| e.into())
    }

    async fn recipe(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::recipes::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;

        crate::recipes::get_recipe_by_id(id, db).await.map_err(|e| e.into())
    }
}

#[Object]
impl RecipesMutations {
    async fn update_recipe(&self, ctx: &Context<'_>, id: i64, recipe: RecipeInput) -> Result<entity::recipes::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;

        crate::recipes::update_recipe(id, recipe, db)
            .await
            .map_err(|e| e.into())
    }

    async fn create_recipe(&self, ctx: &Context<'_>, recipe: RecipeInput) -> Result<entity::recipes::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;
        crate::recipes::create_recipe(recipe, db).await.map_err(|e| e.into())
    }

    async fn delete_recipe(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;

        crate::recipes::delete_recipe(id, db).await.map_err(|e| e.into())
    }
}
