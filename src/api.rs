use async_graphql::{dataloader::DataLoader, extensions::Logger, *};
use sea_orm::DatabaseConnection;

mod ingredients;
mod recipes;
mod session;
mod steps;
mod tags;
mod users;
mod weekplans;

#[derive(async_graphql::MergedObject, Default)]
pub struct MutationRoot(
    session::SessionMutations,
    recipes::RecipesMutations,
    tags::TagsMutations,
    ingredients::IngredientsMutations,
    users::UsersMutations,
    steps::StepsMutations,
    weekplans::WeekplansMutations,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct QueryRoot(
    recipes::RecipesQueries,
    tags::TagsQueries,
    ingredients::IngredientsQueries,
    users::UsersQueries,
    steps::StepsQueries,
    weekplans::WeekplansQueries,
);

pub type RecipesSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(db: DatabaseConnection) -> RecipesSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(DataLoader::new(
            entity::recipes::RecipesLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .data(DataLoader::new(
            entity::steps::StepsLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .data(DataLoader::new(
            entity::steps_ingredients::StepIngredientLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .data(DataLoader::new(
            entity::ingredients::IngredientLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .data(DataLoader::new(
            entity::tags::TagsLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .data(DataLoader::new(
            entity::weekplans::WeekplanLoader { conn: db.clone() },
            tokio::spawn,
        ))
        .extension(Logger)
        .data(db)
        .finish()
}
