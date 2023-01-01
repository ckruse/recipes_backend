use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::*;
use async_graphql::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "weekplans")]
#[graphql(complex, name = "Weekplan")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub date: Date,
    pub recipe_id: i64,
    pub portions: i32,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::recipes::Entity",
        from = "Column::RecipeId",
        to = "super::recipes::Column::Id"
    )]
    Recipe,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    async fn recipe(&self, ctx: &Context<'_>) -> Result<Option<super::recipes::Model>> {
        let loader = ctx.data_unchecked::<DataLoader<WeekplanLoader>>();
        let recipe: Option<super::recipes::Model> = loader.load_one(self.recipe_id).await?;

        Ok(recipe)
    }
}

pub struct WeekplanLoader {
    pub conn: DatabaseConnection,
}

#[async_trait::async_trait]
impl Loader<i64> for WeekplanLoader {
    type Value = super::recipes::Model;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let mut map = HashMap::new();

        let recipes = super::recipes::Entity::find()
            .filter(super::recipes::Column::Id.is_in(keys.to_vec()))
            .all(&self.conn)
            .await?;

        for recipe in recipes {
            map.insert(recipe.id, recipe);
        }

        Ok(map)
    }
}
