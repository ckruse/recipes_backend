use async_graphql::*;
use chrono::{Datelike, NaiveDate, Weekday};
use entity::users::Model as User;
use entity::weekplans as Weekplan;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Condition, Expr, JoinType, Query};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, DbErr, QueryOrder, QuerySelect, TransactionTrait, Unchanged};

pub async fn list_weekplan(
    week: &NaiveDate,
    user: &User,
    db: &DatabaseConnection,
) -> Result<Vec<Weekplan::Model>, DbErr> {
    let week_start = beginning_of_week(week);
    let week_stop = end_of_week(week);

    let weekplan = Weekplan::Entity::find()
        .filter(
            Expr::col(Weekplan::Column::Date)
                .between(week_start, week_stop)
                .and(Expr::col(Weekplan::Column::UserId).eq(user.id)),
        )
        .order_by_asc(Weekplan::Column::Date)
        .order_by_asc(Weekplan::Column::Id)
        .all(db)
        .await?;

    Ok(weekplan)
}

fn beginning_of_week(date: &NaiveDate) -> NaiveDate {
    NaiveDate::from_isoywd_opt(date.iso_week().year(), date.iso_week().week(), Weekday::Mon).unwrap()
}

fn end_of_week(date: &NaiveDate) -> NaiveDate {
    NaiveDate::from_isoywd_opt(date.iso_week().year(), date.iso_week().week(), Weekday::Sun).unwrap()
}

pub async fn create_weekplan_for_week(
    week: NaiveDate,
    user: User,
    tags: Vec<String>,
    db: &DatabaseConnection,
) -> Result<Vec<Weekplan::Model>, DbErr> {
    let week_start = beginning_of_week(&week);
    let week_stop = end_of_week(&week);
    let now = chrono::Utc::now().naive_utc();

    db.transaction::<_, Vec<Weekplan::Model>, DbErr>(|txn| {
        Box::pin(async move {
            let weekplan = Weekplan::Entity::find()
                .filter(
                    Expr::col(Weekplan::Column::Date)
                        .between(week_start, week_stop)
                        .and(Expr::col(Weekplan::Column::UserId).eq(user.id)),
                )
                .order_by_asc(Weekplan::Column::Date)
                .order_by_asc(Weekplan::Column::Id)
                .all(txn)
                .await?;

            if weekplan.is_empty() {
                let mut date = week_start;

                let q = get_random_recipe(user.id, week_start, week_stop, tags);

                while date <= week_stop {
                    let recipe = q.clone().one(txn).await?;

                    if let Some(recipe) = recipe {
                        Weekplan::ActiveModel {
                            date: Set(date),
                            user_id: Set(user.id),
                            recipe_id: Set(recipe.id),
                            portions: Set(2),
                            inserted_at: Set(now),
                            updated_at: Set(now),
                            ..Default::default()
                        }
                        .insert(txn)
                        .await?;
                    }

                    date += chrono::Duration::days(1);
                }
            }

            Weekplan::Entity::find()
                .filter(
                    Expr::col(Weekplan::Column::Date)
                        .between(week_start, week_stop)
                        .and(Expr::col(Weekplan::Column::UserId).eq(user.id)),
                )
                .order_by_asc(Weekplan::Column::Date)
                .order_by_asc(Weekplan::Column::Id)
                .all(txn)
                .await
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

pub fn get_random_recipe(
    user_id: i64,
    week_start: NaiveDate,
    week_stop: NaiveDate,
    tags: Vec<String>,
) -> Select<entity::recipes::Entity> {
    entity::recipes::Entity::find()
        .filter(
            Expr::col(entity::recipes::Column::Id).not_in_subquery(
                Query::select()
                    .column(Weekplan::Column::RecipeId)
                    .from(Weekplan::Entity)
                    .and_where(
                        Expr::col(Weekplan::Column::UserId)
                            .eq(user_id)
                            .and(Expr::col(Weekplan::Column::Date).between(week_start, week_stop)),
                    )
                    .to_owned(),
            ),
        )
        .filter(
            Expr::col(entity::recipes::Column::Id).in_subquery(
                Query::select()
                    .column(entity::recipes_tags::Column::RecipeId)
                    .from(entity::recipes_tags::Entity)
                    .join(
                        JoinType::InnerJoin,
                        entity::tags::Entity,
                        Condition::all().add(
                            Expr::col(entity::tags::Column::Id)
                                .eq(Expr::col(entity::recipes_tags::Column::TagId))
                                .and(Expr::col(entity::tags::Column::Name).is_in(tags)),
                        ),
                    )
                    .to_owned(),
            ),
        )
        .limit(1)
        .order_by_asc(Expr::cust("RANDOM()"))
}

pub async fn get_weekplan_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<Weekplan::Model>, DbErr> {
    entity::weekplans::Entity::find_by_id(id).one(db).await
}

pub async fn delete_weekplan(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    let deleted = Weekplan::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1;
    Ok(deleted)
}

pub async fn replace_weekplan_recipe(
    weekplan: Weekplan::Model,
    tags: Vec<String>,
    db: &DatabaseConnection,
) -> Result<Weekplan::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let start = beginning_of_week(&weekplan.date);
    let stop = end_of_week(&weekplan.date);

    let q = get_random_recipe(weekplan.user_id, start, stop, tags);
    let recipe = q.one(db).await?.ok_or(DbErr::Query(sea_orm::RuntimeErr::Internal(
        "No recipe found".to_owned(),
    )))?;

    let weekplan = Weekplan::ActiveModel {
        id: Unchanged(weekplan.id),
        recipe_id: Set(recipe.id),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(weekplan)
}
