//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.4

use async_graphql::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(2))")]
pub enum Units {
    #[sea_orm(string_value = "pcs")]
    PCS,
    #[sea_orm(string_value = "tbsp")]
    TBSP,
    #[sea_orm(string_value = "tsp")]
    TSP,
    #[sea_orm(string_value = "skosh")]
    SKOSH,
    #[sea_orm(string_value = "pinch")]
    PINCH,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "ingredient_units")]
#[graphql(concrete(name = "IngredientUnit", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub ingredient_id: i64,
    pub identifier: Units,
    pub base_value: f64,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ingredients::Entity",
        from = "Column::IngredientId",
        to = "super::ingredients::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Ingredients,
    #[sea_orm(has_many = "super::steps_ingredients::Entity")]
    StepsIngridients,
}

impl Related<super::ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ingredients.def()
    }
}

impl Related<super::steps_ingredients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StepsIngridients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
