use async_graphql::{dataloader::DataLoader, extensions::Logger, *};
use sea_orm::DatabaseConnection;

mod ingredients;
mod recipes;
mod session;
mod steps;
mod tags;
mod users;

#[derive(async_graphql::MergedObject, Default)]
pub struct MutationRoot(
    session::SessionMutations,
    recipes::RecipesMutations,
    tags::TagsMutations,
    ingredients::IngredientsMutations,
    users::UsersMutations,
    steps::StepsMutations,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct QueryRoot(
    recipes::RecipesQueries,
    tags::TagsQueries,
    ingredients::IngredientsQueries,
    users::UsersQueries,
    steps::StepsQueries,
);

pub type RecipesSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(db: DatabaseConnection) -> RecipesSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(DataLoader::new(
            entity::recipes::RecipesLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .data(DataLoader::new(
            entity::steps::StepsLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .data(DataLoader::new(
            entity::steps_ingridients::StepIngredientLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .data(DataLoader::new(
            entity::ingredients::IngredientLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .data(DataLoader::new(
            entity::tags::TagsLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .extension(Logger)
        .data(db)
        .finish()
}
