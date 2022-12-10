use async_graphql::*;
use chrono::Utc;
use entity::steps::Model;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};

pub async fn list_steps(recipe_id: i64, db: &DatabaseConnection) -> Result<Vec<entity::steps::Model>, DbErr> {
    entity::steps::Entity::find()
        .filter(entity::steps::Column::RecipeId.eq(recipe_id))
        .all(db)
        .await
}

pub async fn count_steps(recipe_id: i64, db: &DatabaseConnection) -> Result<u64, DbErr> {
    entity::steps::Entity::find()
        .filter(entity::steps::Column::RecipeId.eq(recipe_id))
        .count(db)
        .await
}

pub async fn get_step_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<entity::steps::Model>, DbErr> {
    entity::steps::Entity::find_by_id(id).one(db).await
}

#[derive(SimpleObject, InputObject)]
pub struct StepIngredientInput {
    pub id: Option<i64>,
    pub ingredient_id: i64,
    pub amount: f64,
    pub annotation: Option<String>,
    pub unit_id: Option<i64>,
}

#[derive(SimpleObject, InputObject)]
pub struct StepInput {
    pub position: i32,
    pub description: Option<String>,
    pub preparation_time: i32,
    pub cooking_time: i32,
    pub step_ingredients: Vec<StepIngredientInput>,
}

pub async fn create_step(
    recipe_id: i64,
    step_values: &StepInput,
    db: &DatabaseConnection,
) -> Result<entity::steps::Model, DbErr> {
    let now = Utc::now().naive_utc();

    let step = entity::steps::ActiveModel {
        recipe_id: Set(recipe_id),
        position: Set(step_values.position),
        description: Set(step_values.description.clone()),
        preparation_time: Set(step_values.preparation_time),
        cooking_time: Set(step_values.cooking_time),
        inserted_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let step = step.insert(db).await?;

    for step_ingredient in &step_values.step_ingredients {
        let step_ingredient = entity::steps_ingridients::ActiveModel {
            step_id: Set(step.id),
            ingredient_id: Set(step_ingredient.ingredient_id),
            amount: Set(step_ingredient.amount),
            annotation: Set(step_ingredient.annotation.clone()),
            unit_id: Set(step_ingredient.unit_id),
            inserted_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        step_ingredient.save(db).await?;
    }

    Ok(step)
}

pub async fn update_step(
    step_id: i64,
    step_values: StepInput,
    db: &DatabaseConnection,
) -> Result<entity::steps::Model, DbErr> {
    let step = entity::steps::ActiveModel {
        id: Set(step_id),
        position: Set(step_values.position),
        description: Set(step_values.description.clone()),
        preparation_time: Set(step_values.preparation_time),
        cooking_time: Set(step_values.cooking_time),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    db.transaction::<_, entity::steps::Model, DbErr>(|txn| {
        Box::pin(async move {
            let step = step.insert(txn).await?;

            let existing_ingredients = entity::steps_ingridients::Entity::find()
                .filter(entity::steps_ingridients::Column::StepId.eq(step_id))
                .all(txn)
                .await?;

            let ingredients_to_delete = existing_ingredients
                .iter()
                .filter(|existing_ingredient| {
                    step_values
                        .step_ingredients
                        .iter()
                        .any(|step_ingredient| step_ingredient.id == Some(existing_ingredient.id))
                })
                .collect::<Vec<&entity::steps_ingridients::Model>>();

            for ingredient in ingredients_to_delete {
                entity::recipes::Entity::delete_by_id(ingredient.id).exec(txn).await?;
            }

            for step_ingredient_values in &step_values.step_ingredients {
                let mut step_ingredient = entity::steps_ingridients::ActiveModel {
                    step_id: Set(step.id),
                    ingredient_id: Set(step_ingredient_values.ingredient_id),
                    amount: Set(step_ingredient_values.amount),
                    annotation: Set(step_ingredient_values.annotation.clone()),
                    unit_id: Set(step_ingredient_values.unit_id),
                    ..Default::default()
                };

                if let Some(id) = step_ingredient_values.id {
                    step_ingredient.id = Set(id);
                }

                step_ingredient.save(txn).await?;
            }

            Ok(step)
        })
    })
    .await
    .map_err(|_e| DbErr::Query(sea_orm::RuntimeErr::Internal("Transaction failed".to_string())))
}

pub async fn delete_step(step: Model, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(step.delete(db).await?.rows_affected == 1)
}
