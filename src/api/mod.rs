use async_graphql::{dataloader::DataLoader, *};
use sea_orm::DatabaseConnection;

mod ingredients;
mod recipes;
mod session;
mod tags;
mod users;

#[derive(async_graphql::MergedObject, Default)]
pub struct MutationRoot(
    session::SessionMutations,
    recipes::RecipesMutations,
    tags::TagsMutations,
    ingredients::IngredientsMutations,
);

#[derive(async_graphql::MergedObject, Default)]
pub struct QueryRoot(
    recipes::RecipesQueries,
    tags::TagsQueries,
    ingredients::IngredientsQueries,
);

pub type RecipesSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(db: DatabaseConnection) -> RecipesSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(DataLoader::new(
            entity::recipes::TagsLoader { conn: db.clone() },
            actix_web::rt::spawn,
        ))
        .data(db)
        .finish()
}
