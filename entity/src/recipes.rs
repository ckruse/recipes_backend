use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::*;
use async_graphql::*;

use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, FromQueryResult, JoinType, QuerySelect};

use serde::{Deserialize, Serialize};

use crate::{recipes_tags, tags};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "recipes")]
#[graphql(complex, concrete(name = "Recipe", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::steps::Entity")]
    Steps,
    #[sea_orm(has_many = "super::recipes_tags::Entity")]
    RecipesTags,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::OwnerId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Owner,
}

impl Related<super::steps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Steps.def()
    }
}

impl Related<super::recipes_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecipesTags.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<tags::Model>> {
        let loader = ctx.data_unchecked::<DataLoader<TagsLoader>>();
        let name: Option<Vec<tags::Model>> = loader.load_one(self.id).await?;
        name.ok_or_else(|| "Not found".into())
    }
}

pub struct TagsLoader {
    pub conn: DatabaseConnection,
}

#[derive(FromQueryResult)]
struct RecipeIdAndTag {
    pub recipe_id: i64,
    pub tag: String,
    pub id: i64,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[async_trait::async_trait]
impl Loader<i64> for TagsLoader {
    type Value = Vec<tags::Model>;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let tags = tags::Entity::find()
            .join(JoinType::InnerJoin, tags::Relation::RecipesTags.def())
            .column_as(recipes_tags::Column::RecipeId, "recipe_id")
            .filter(recipes_tags::Column::RecipeId.is_in(keys.to_vec()))
            .into_model::<RecipeIdAndTag>()
            .all(&self.conn)
            .await?;

        let map = tags
            .into_iter()
            .group_by(|tag| tag.recipe_id)
            .into_iter()
            .map(|(key, group)| {
                let tags = group
                    .into_iter()
                    .map(|tag| tags::Model {
                        id: tag.id,
                        tag: tag.tag,
                        inserted_at: tag.inserted_at,
                        updated_at: tag.updated_at,
                    })
                    .collect();

                (key, tags)
            })
            .collect();

        Ok(map)
    }
}
