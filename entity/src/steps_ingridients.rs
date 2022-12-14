//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.4
use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::*;
use async_graphql::*;
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{ingredient_units, ingredients};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "steps_ingridients")]
#[graphql(complex, concrete(name = "StepIngredient", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub step_id: i64,
    pub ingredient_id: i64,
    pub amount: Option<f64>,
    pub annotation: Option<String>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    pub unit_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ingredient_units::Entity",
        from = "Column::UnitId",
        to = "super::ingredient_units::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    IngredientUnits,
    #[sea_orm(
        belongs_to = "super::ingredients::Entity",
        from = "Column::IngredientId",
        to = "super::ingredients::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Ingredients,
    #[sea_orm(
        belongs_to = "super::steps::Entity",
        from = "Column::StepId",
        to = "super::steps::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Steps,
}

impl Related<super::ingredient_units::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IngredientUnits.def()
    }
}

impl Related<super::ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ingredients.def()
    }
}

impl Related<super::steps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Steps.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct StepIngredientLoader {
    pub conn: DatabaseConnection,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct UnitId(pub i64);
#[derive(Clone, Eq, PartialEq, Hash)]
struct IngredientId(pub i64);

#[ComplexObject]
impl Model {
    async fn unit(&self, ctx: &Context<'_>) -> Result<Option<ingredient_units::Model>> {
        if let Some(unit_id) = self.unit_id {
            let loader = ctx.data_unchecked::<DataLoader<StepIngredientLoader>>();
            let unit = loader.load_one(UnitId(unit_id)).await?;
            Ok(unit)
        } else {
            Ok(None)
        }
    }

    async fn ingredient(&self, ctx: &Context<'_>) -> Result<ingredients::Model> {
        let loader = ctx.data_unchecked::<DataLoader<StepIngredientLoader>>();
        let ingredient = loader.load_one(IngredientId(self.ingredient_id)).await?;
        ingredient.ok_or_else(|| "Not found".into())
    }
}

#[async_trait::async_trait]
impl Loader<UnitId> for StepIngredientLoader {
    type Value = ingredient_units::Model;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[UnitId]) -> Result<HashMap<UnitId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|id| id.0).collect_vec();

        let units = ingredient_units::Entity::find()
            .filter(ingredient_units::Column::Id.is_in(ids.to_vec()))
            .into_model::<ingredient_units::Model>()
            .all(&self.conn)
            .await?;

        let map = units
            .into_iter()
            .map(|group| (UnitId(group.id), group))
            .into_iter()
            .collect();

        Ok(map)
    }
}

#[async_trait::async_trait]
impl Loader<IngredientId> for StepIngredientLoader {
    type Value = ingredients::Model;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[IngredientId]) -> Result<HashMap<IngredientId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|id| id.0).collect_vec();

        let units = ingredients::Entity::find()
            .filter(ingredients::Column::Id.is_in(ids.to_vec()))
            .into_model::<ingredients::Model>()
            .all(&self.conn)
            .await?;

        let map = units
            .into_iter()
            .map(|ingredient| (IngredientId(ingredient.id), ingredient))
            .into_iter()
            .collect();

        Ok(map)
    }
}
