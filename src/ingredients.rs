use async_graphql::*;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{DatabaseConnection, QuerySelect};

#[derive(SimpleObject, InputObject)]
pub struct IngredientInput {
    name: String,
    reference: entity::ingredients::Reference,
    carbs: f64,
    fat: f64,
    proteins: f64,
    alc: f64,
}

pub async fn list_ingredients(
    limit: u64,
    offset: u64,
    search: Option<String>,
    db: &DatabaseConnection,
) -> Result<Vec<entity::ingredients::Model>> {
    let mut query = entity::ingredients::Entity::find().limit(limit).offset(offset);

    if let Some(search) = search {
        query = query.filter(entity::ingredients::Column::Name.contains(&search));
    }

    let ingredients = query.all(db).await?;
    Ok(ingredients)
}

pub async fn count_ingredients(search: Option<String>, db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut query = entity::ingredients::Entity::find();

    if let Some(search) = search {
        query = query.filter(entity::ingredients::Column::Name.contains(&search));
    }

    query.count(db).await
}

pub async fn get_ingredient_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<entity::ingredients::Model>> {
    let ingredient = entity::ingredients::Entity::find_by_id(id).one(db).await?;
    Ok(ingredient)
}

pub async fn create_ingredient(
    ingredient_values: IngredientInput,
    db: &DatabaseConnection,
) -> Result<entity::ingredients::Model, DbErr> {
    let now = Utc::now().naive_utc();

    entity::ingredients::ActiveModel {
        name: Set(ingredient_values.name),
        reference: Set(ingredient_values.reference),
        carbs: Set(ingredient_values.carbs),
        fat: Set(ingredient_values.fat),
        proteins: Set(ingredient_values.proteins),
        alc: Set(ingredient_values.alc),
        inserted_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn update_ingredient(
    id: i64,
    ingredient_values: IngredientInput,
    db: &DatabaseConnection,
) -> Result<entity::ingredients::Model, DbErr> {
    let now = Utc::now().naive_utc();

    entity::ingredients::ActiveModel {
        id: Unchanged(id),
        name: Set(ingredient_values.name),
        reference: Set(ingredient_values.reference),
        carbs: Set(ingredient_values.carbs),
        fat: Set(ingredient_values.fat),
        proteins: Set(ingredient_values.proteins),
        alc: Set(ingredient_values.alc),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await
}

pub async fn delete_ingredient(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(entity::ingredients::Entity::delete_by_id(id)
        .exec(db)
        .await?
        .rows_affected
        == 1)
}
