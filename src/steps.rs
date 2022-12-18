use async_graphql::*;
use chrono::Utc;
use entity::steps::Model;
use migration::Order;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, DbErr, QueryOrder, QuerySelect, TransactionTrait, Unchanged};

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
    pub amount: Option<f64>,
    pub annotation: Option<String>,
    pub unit_id: Option<i64>,
}

#[derive(SimpleObject, InputObject)]
pub struct StepInput {
    pub position: i32,
    #[graphql(validator(max_length = 12288))]
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
    let now = Utc::now().naive_utc();

    let step = entity::steps::ActiveModel {
        id: Set(step_id),
        position: Set(step_values.position),
        description: Set(step_values.description.clone()),
        preparation_time: Set(step_values.preparation_time),
        cooking_time: Set(step_values.cooking_time),
        updated_at: Set(now),
        ..Default::default()
    };

    db.transaction::<_, entity::steps::Model, DbErr>(|txn| {
        Box::pin(async move {
            let step = step.update(txn).await?;

            let existing_ingredients = entity::steps_ingridients::Entity::find()
                .filter(entity::steps_ingridients::Column::StepId.eq(step_id))
                .all(txn)
                .await?;

            let ingredients_to_delete = existing_ingredients
                .iter()
                .filter(|existing_ingredient| {
                    !step_values
                        .step_ingredients
                        .iter()
                        .any(|step_ingredient| step_ingredient.id == Some(existing_ingredient.id))
                })
                .collect::<Vec<&entity::steps_ingridients::Model>>();

            let ids = ingredients_to_delete.iter().map(|i| i.id).collect::<Vec<i64>>();
            entity::steps_ingridients::Entity::delete_many()
                .filter(entity::steps_ingridients::Column::Id.is_in(ids))
                .exec(txn)
                .await?;

            for step_ingredient_values in &step_values.step_ingredients {
                let mut step_ingredient = entity::steps_ingridients::ActiveModel {
                    step_id: Set(step.id),
                    ingredient_id: Set(step_ingredient_values.ingredient_id),
                    amount: Set(step_ingredient_values.amount),
                    annotation: Set(step_ingredient_values.annotation.clone()),
                    unit_id: Set(step_ingredient_values.unit_id),
                    updated_at: Set(now),
                    ..Default::default()
                };

                if let Some(id) = step_ingredient_values.id {
                    step_ingredient.id = Unchanged(id);
                } else {
                    step_ingredient.inserted_at = Set(now);
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

pub async fn move_step_up(step: Model, db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    if step.position < 1 {
        return Ok(vec![step]);
    }

    let other_step = entity::steps::Entity::find()
        .filter(entity::steps::Column::RecipeId.eq(step.recipe_id))
        .filter(entity::steps::Column::Position.lt(step.position))
        .limit(1)
        .order_by(entity::steps::Column::Position, Order::Desc)
        .one(db)
        .await?;

    let mut result: Vec<Model> = vec![];

    if let Some(other_step) = other_step {
        let active_model = entity::steps::ActiveModel {
            id: Unchanged(other_step.id),
            position: Set(other_step.position + 1),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let rslt = active_model.update(db).await?;
        result.push(rslt);
    }

    let active_model = entity::steps::ActiveModel {
        id: Unchanged(step.id),
        position: Set(step.position - 1),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let rslt = active_model.update(db).await?;

    result.push(rslt);

    Ok(result)
}

pub async fn move_step_down(step: Model, db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    let other_step = entity::steps::Entity::find()
        .filter(entity::steps::Column::RecipeId.eq(step.recipe_id))
        .filter(entity::steps::Column::Position.gt(step.position))
        .limit(1)
        .order_by(entity::steps::Column::Position, Order::Asc)
        .one(db)
        .await?;

    if other_step.is_none() {
        return Ok(vec![step]);
    }

    let mut result: Vec<Model> = vec![];
    let other_step = other_step.unwrap();

    let active_model = entity::steps::ActiveModel {
        id: Unchanged(other_step.id),
        position: Set(other_step.position - 1),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let rslt = active_model.update(db).await?;
    result.push(rslt);

    let active_model = entity::steps::ActiveModel {
        id: Unchanged(step.id),
        position: Set(step.position + 1),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let rslt = active_model.update(db).await?;

    result.push(rslt);

    Ok(result)
}
