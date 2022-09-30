//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use async_graphql::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(2))")]
pub enum Reference {
    #[sea_orm(string_value = "g")]
    G,
    #[sea_orm(string_value = "ml")]
    ML,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "ingredients")]
#[graphql(concrete(name = "Ingredient", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub reference: Reference,
    pub carbs: f32,
    pub fat: f32,
    pub proteins: f32,
    pub alc: f32,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::steps_ingridients::Entity")]
    StepsIngridients,
}

impl Related<super::steps_ingridients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StepsIngridients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
