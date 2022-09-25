use async_graphql::*;
use sea_orm::DatabaseConnection;

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
        let db = ctx.data::<DatabaseConnection>()?;
        crate::ingredients::list_ingredients(limit, offset, search, db)
            .await
            .map_err(|e| e.into())
    }

    pub async fn count_ingredients(&self, ctx: &Context<'_>, search: Option<String>) -> Result<usize> {
        let db = ctx.data::<DatabaseConnection>()?;
        crate::ingredients::count_ingredients(search, db)
            .await
            .map_err(|e| e.into())
    }

    async fn ingredient(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::ingredients::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        crate::ingredients::get_ingredient_by_id(id, db)
            .await
            .map_err(|e| e.into())
    }
}

#[Object]
impl IngredientsMutations {
    async fn create_ingredient(
        &self,
        ctx: &Context<'_>,
        ingredient: IngredientInput,
    ) -> Result<entity::ingredients::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;

        crate::ingredients::create_ingredient(ingredient, db)
            .await
            .map_err(|e| e.into())
    }

    async fn update_ingredient(
        &self,
        ctx: &Context<'_>,
        id: i64,
        ingredient: IngredientInput,
    ) -> Result<entity::ingredients::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;

        crate::ingredients::update_ingredient(id, ingredient, db)
            .await
            .map_err(|e| e.into())
    }

    async fn delete_ingredient(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;

        crate::ingredients::delete_ingredient(id, db)
            .await
            .map_err(|e| e.into())
    }
}
