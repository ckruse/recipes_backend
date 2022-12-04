//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.4
use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::*;
use async_graphql::*;
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::steps_ingridients;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "steps")]
#[graphql(complex, concrete(name = "Step", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub recipe_id: i64,
    pub position: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    pub preparation_time: i32,
    pub cooking_time: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::recipes::Entity",
        from = "Column::RecipeId",
        to = "super::recipes::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Recipes,
    #[sea_orm(has_many = "super::steps_ingridients::Entity")]
    StepsIngridients,
}

impl Related<super::recipes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Recipes.def()
    }
}

impl Related<super::steps_ingridients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StepsIngridients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct StepsLoader {
    pub conn: DatabaseConnection,
}

#[ComplexObject]
impl Model {
    async fn step_ingredients(&self, ctx: &Context<'_>) -> Result<Vec<steps_ingridients::Model>> {
        let loader = ctx.data_unchecked::<DataLoader<StepsLoader>>();
        let steps: Option<Vec<steps_ingridients::Model>> = loader.load_one(self.id).await?;
        Ok(steps.unwrap_or(vec![]))
    }
}

#[async_trait::async_trait]
impl Loader<i64> for StepsLoader {
    type Value = Vec<steps_ingridients::Model>;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let steps = steps_ingridients::Entity::find()
            .filter(steps_ingridients::Column::StepId.is_in(keys.to_vec()))
            .into_model::<steps_ingridients::Model>()
            .all(&self.conn)
            .await?;

        let map = steps
            .into_iter()
            .group_by(|step| step.step_id)
            .into_iter()
            .map(|(key, group)| (key, group.collect()))
            .collect();

        Ok(map)
    }
}
