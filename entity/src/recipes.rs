//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.4

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;

use async_graphql::dataloader::*;
use async_graphql::*;
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, FromQueryResult, JoinType, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::{fitting, ingredient_units, ingredients, recipes_tags, steps, steps_ingredients, tags};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "recipes")]
#[graphql(complex, name = "Recipe")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub default_servings: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub owner_id: Option<i64>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    #[graphql(skip)]
    pub image: Option<String>,
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
        on_delete = "SetNull"
    )]
    Users,
    #[sea_orm(has_many = "super::fitting::Entity")]
    Fitting,
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
        Relation::Users.def()
    }
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        super::recipes_tags::Relation::Tags.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::recipes_tags::Relation::Recipes.def().rev())
    }
}

impl Related<super::fitting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fitting.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct TagId(pub i64);
#[derive(Clone, Eq, PartialEq, Hash)]
struct StepId(i64);
#[derive(Clone, Eq, PartialEq, Hash)]
struct FittingRecipesId(i64);

#[derive(Clone, Eq, PartialEq, Hash)]
struct CaloriesId(i64);

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct RecipeImage {
    pub thumb: String,
    pub large: String,
    pub original: String,
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

#[derive(Clone, Debug, Serialize, SimpleObject)]
struct CaloriesResult {
    proteins: f64,
    carbs: f64,
    fats: f64,
    alcohol: f64,
    calories: f64,
}

#[ComplexObject]
impl Model {
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<tags::Model>> {
        let loader = ctx.data_unchecked::<DataLoader<RecipesLoader>>();
        let name: Option<Vec<tags::Model>> = loader.load_one(TagId(self.id)).await?;
        Ok(name.unwrap_or_default())
    }

    async fn steps(&self, ctx: &Context<'_>) -> Result<Vec<steps::Model>> {
        let loader = ctx.data_unchecked::<DataLoader<RecipesLoader>>();
        let steps: Option<Vec<steps::Model>> = loader.load_one(StepId(self.id)).await?;
        Ok(steps.unwrap_or_default())
    }

    async fn fitting_recipes(&self, ctx: &Context<'_>) -> Result<Vec<Model>> {
        let loader = ctx.data_unchecked::<DataLoader<RecipesLoader>>();
        let fitting_recipes: Option<Vec<Model>> = loader.load_one(FittingRecipesId(self.id)).await?;
        Ok(fitting_recipes.unwrap_or_default())
    }

    async fn image(&self, _ctx: &Context<'_>) -> Option<RecipeImage> {
        let ext = get_extension_from_filename(self.image.as_ref()?).or(Some("jpg"))?;

        self.image.as_ref().map(|_| RecipeImage {
            thumb: format!("/pictures/{}/thumbnail.{}", self.id, ext),
            large: format!("/pictures/{}/large.{}", self.id, ext),
            original: format!("/pictures/{}/original.{}", self.id, ext),
        })
    }

    async fn calories(&self, ctx: &Context<'_>) -> Result<Option<CaloriesResult>> {
        let loader = ctx.data_unchecked::<DataLoader<RecipesLoader>>();
        let calories = loader.load_one(CaloriesId(self.id)).await?;

        Ok(calories)
    }
}

pub struct RecipesLoader {
    pub conn: DatabaseConnection,
}

#[derive(FromQueryResult, Debug)]
struct RecipeIdAndTag {
    pub recipe_id: i64,
    pub name: String,
    pub id: i64,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
}

#[async_trait::async_trait]
impl Loader<TagId> for RecipesLoader {
    type Value = Vec<tags::Model>;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[TagId]) -> Result<HashMap<TagId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|k| k.0).collect_vec();
        let tags = tags::Entity::find()
            .join(JoinType::InnerJoin, tags::Relation::RecipesTags.def())
            .column_as(recipes_tags::Column::RecipeId, "recipe_id")
            .filter(recipes_tags::Column::RecipeId.is_in(ids))
            .order_by_asc(recipes_tags::Column::RecipeId)
            .order_by_asc(tags::Column::Name)
            .into_model::<RecipeIdAndTag>()
            .all(&self.conn)
            .await?;

        let map = tags
            .into_iter()
            .group_by(|tag| tag.recipe_id)
            .into_iter()
            .map(|(recipe_id, group)| {
                let tags = group
                    .into_iter()
                    .map(|tag| tags::Model {
                        id: tag.id,
                        name: tag.name,
                        inserted_at: tag.inserted_at,
                        updated_at: tag.updated_at,
                    })
                    .collect();

                (TagId(recipe_id), tags)
            })
            .collect();

        Ok(map)
    }
}

#[async_trait::async_trait]
impl Loader<StepId> for RecipesLoader {
    type Value = Vec<steps::Model>;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[StepId]) -> Result<HashMap<StepId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|k| k.0).collect_vec();

        let steps = steps::Entity::find()
            .filter(steps::Column::RecipeId.is_in(ids))
            .order_by_asc(steps::Column::RecipeId)
            .order_by_asc(steps::Column::Position)
            .into_model::<steps::Model>()
            .all(&self.conn)
            .await?;

        let map = steps
            .into_iter()
            .group_by(|step| step.recipe_id)
            .into_iter()
            .map(|(key, group)| (StepId(key), group.collect()))
            .collect();

        Ok(map)
    }
}

#[derive(FromQueryResult, Debug)]
struct RecipeIdAndRecipe {
    pub recipe_id: i64,
    pub id: i64,
    pub name: String,
    pub default_servings: i32,
    pub description: Option<String>,
    pub owner_id: Option<i64>,
    pub inserted_at: DateTime,
    pub updated_at: DateTime,
    pub image: Option<String>,
}

#[async_trait::async_trait]
impl Loader<FittingRecipesId> for RecipesLoader {
    type Value = Vec<Model>;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[FittingRecipesId]) -> Result<HashMap<FittingRecipesId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|k| k.0).collect_vec();

        let fitting_recipes = Entity::find()
            .join(
                JoinType::InnerJoin,
                Entity::has_many(fitting::Entity)
                    .from(Column::Id)
                    .to(fitting::Column::FittingRecipeId)
                    .into(),
            )
            .column_as(fitting::Column::RecipeId, "recipe_id")
            .filter(fitting::Column::RecipeId.is_in(ids))
            .order_by_asc(fitting::Column::RecipeId)
            .into_model::<RecipeIdAndRecipe>()
            .all(&self.conn)
            .await?;

        let map = fitting_recipes
            .into_iter()
            .group_by(|recipe| recipe.recipe_id)
            .into_iter()
            .map(|(key, group)| {
                let recipes = group
                    .into_iter()
                    .map(|recipe| Model {
                        id: recipe.id,
                        name: recipe.name,
                        default_servings: recipe.default_servings,
                        description: recipe.description,
                        owner_id: recipe.owner_id,
                        inserted_at: recipe.inserted_at,
                        updated_at: recipe.updated_at,
                        image: recipe.image,
                    })
                    .collect();

                (FittingRecipesId(key), recipes)
            })
            .collect();

        Ok(map)
    }
}

#[derive(FromQueryResult, Debug)]
struct RecipeIdAndCalories {
    recipe_id: i64,
    carbs: f64,
    fat: f64,
    proteins: f64,
    alc: f64,
    base_value: Option<f64>,
    amount: Option<f64>,
}

#[async_trait::async_trait]
impl Loader<CaloriesId> for RecipesLoader {
    type Value = CaloriesResult;
    type Error = Arc<sea_orm::error::DbErr>;

    async fn load(&self, keys: &[CaloriesId]) -> Result<HashMap<CaloriesId, Self::Value>, Self::Error> {
        let ids = keys.iter().map(|k| k.0).collect_vec();

        let calories = steps_ingredients::Entity::find()
            .join(JoinType::InnerJoin, steps_ingredients::Relation::Steps.def())
            .join(JoinType::InnerJoin, steps_ingredients::Relation::Ingredients.def())
            .join(JoinType::LeftJoin, steps_ingredients::Relation::IngredientUnits.def())
            .select_only()
            .column_as(steps::Column::RecipeId, "recipe_id")
            .column_as(ingredients::Column::Carbs, "carbs")
            .column_as(ingredients::Column::Fat, "fat")
            .column_as(ingredients::Column::Proteins, "proteins")
            .column_as(ingredients::Column::Alc, "alc")
            .column_as(ingredient_units::Column::BaseValue, "base_value")
            .column_as(steps_ingredients::Column::Amount, "amount")
            .filter(steps::Column::RecipeId.is_in(ids))
            .filter(steps_ingredients::Column::Amount.is_not_null())
            .order_by_asc(steps::Column::RecipeId)
            .into_model::<RecipeIdAndCalories>()
            .all(&self.conn)
            .await?;

        let map: HashMap<CaloriesId, CaloriesResult> = calories
            .into_iter()
            .group_by(|step| step.recipe_id)
            .into_iter()
            .map(|(key, group)| {
                let calories = group.into_iter().fold(
                    CaloriesResult {
                        carbs: 0.0,
                        fats: 0.0,
                        proteins: 0.0,
                        alcohol: 0.0,
                        calories: 0.0,
                    },
                    |mut acc, row| {
                        let amount = row.amount.unwrap();
                        let grams = row.base_value.map(|bv| bv * amount).unwrap_or(amount) / 100.0;

                        acc.carbs += row.carbs * grams;
                        acc.fats += row.fat * grams;
                        acc.proteins += row.proteins * grams;
                        acc.alcohol += row.alc * grams;
                        acc.calories = acc.carbs * 4.1 + acc.fats * 9.3 + acc.proteins * 4.1 + acc.alcohol * 7.1;

                        acc
                    },
                );

                (CaloriesId(key), calories)
            })
            .collect();

        Ok(map)
    }
}
