use async_graphql::*;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{Condition, DatabaseConnection, QueryOrder, QuerySelect, TransactionTrait};

#[derive(SimpleObject, InputObject)]
pub struct UnitInput {
    id: Option<i64>,
    identifier: entity::ingredient_units::Units,
    base_value: f64,
}

#[derive(SimpleObject, InputObject)]
pub struct IngredientInput {
    #[graphql(validator(chars_min_length = 3, chars_max_length = 255))]
    name: String,
    reference: entity::ingredients::Reference,
    carbs: f64,
    fat: f64,
    proteins: f64,
    alc: f64,
    units: Option<Vec<UnitInput>>,
}

pub async fn list_ingredients(
    limit: u64,
    offset: u64,
    search: Option<Vec<String>>,
    db: &DatabaseConnection,
) -> Result<Vec<entity::ingredients::Model>> {
    let mut query = entity::ingredients::Entity::find().limit(limit).offset(offset);

    if let Some(search) = search {
        let mut cond = Condition::all();

        for s in search {
            cond = cond.add(
                Expr::expr(Func::lower(Expr::col((
                    entity::ingredients::Entity,
                    entity::ingredients::Column::Name,
                ))))
                .like(format!("%{}%", s)),
            );
        }

        query = query.filter(cond);
    }

    query
        .order_by_asc(entity::ingredients::Column::Name)
        .all(db)
        .await
        .map_err(|e| e.into())
}

pub async fn count_ingredients(search: Option<Vec<String>>, db: &DatabaseConnection) -> Result<u64> {
    let mut query = entity::ingredients::Entity::find();

    if let Some(search) = search {
        let mut cond = Condition::all();

        for s in search {
            cond = cond.add(
                Expr::expr(Func::lower(Expr::col((
                    entity::ingredients::Entity,
                    entity::ingredients::Column::Name,
                ))))
                .like(format!("%{}%", s)),
            );
        }

        query = query.filter(cond);
    }

    query.count(db).await.map_err(|e| e.into())
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

    db.transaction::<_, entity::ingredients::Model, DbErr>(|txn| {
        Box::pin(async move {
            let ingredient = entity::ingredients::ActiveModel {
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
            .insert(txn)
            .await?;

            if let Some(units) = ingredient_values.units {
                for unit in units {
                    entity::ingredient_units::ActiveModel {
                        ingredient_id: Set(ingredient.id),
                        identifier: Set(unit.identifier),
                        base_value: Set(unit.base_value),
                        inserted_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                }
            }

            Ok(ingredient)
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

pub async fn update_ingredient(
    id: i64,
    ingredient_values: IngredientInput,
    db: &DatabaseConnection,
) -> Result<entity::ingredients::Model, DbErr> {
    let now = Utc::now().naive_utc();

    let values = entity::ingredients::ActiveModel {
        id: Unchanged(id),
        name: Set(ingredient_values.name),
        reference: Set(ingredient_values.reference),
        carbs: Set(ingredient_values.carbs),
        fat: Set(ingredient_values.fat),
        proteins: Set(ingredient_values.proteins),
        alc: Set(ingredient_values.alc),
        updated_at: Set(now),
        ..Default::default()
    };

    db.transaction::<_, entity::ingredients::Model, DbErr>(|txn| {
        Box::pin(async move {
            let ingredient = values.update(txn).await?;

            if let Some(units) = ingredient_values.units {
                let unit_ids = units.iter().filter_map(|unit| unit.id).collect::<Vec<i64>>();

                entity::ingredient_units::Entity::delete_many()
                    .filter(entity::ingredient_units::Column::Id.is_not_in(unit_ids.clone()))
                    .filter(entity::ingredient_units::Column::IngredientId.eq(ingredient.id))
                    .exec(txn)
                    .await?;

                for unit in units {
                    let mut unit_values = entity::ingredient_units::ActiveModel {
                        ingredient_id: Set(id),
                        identifier: Set(unit.identifier),
                        base_value: Set(unit.base_value),
                        updated_at: Set(now),
                        ..Default::default()
                    };

                    if let Some(id) = unit.id {
                        unit_values.id = Set(id);
                        unit_values.update(txn).await?;
                    } else {
                        unit_values.inserted_at = Set(now);
                        unit_values.insert(txn).await?;
                    }
                }
            }

            Ok(ingredient)
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

pub async fn delete_ingredient(id: i64, db: &DatabaseConnection) -> Result<bool> {
    Ok(entity::ingredients::Entity::delete_by_id(id)
        .exec(db)
        .await?
        .rows_affected
        == 1)
}
