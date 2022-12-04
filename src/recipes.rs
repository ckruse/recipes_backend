use async_graphql::*;
use chrono::Utc;
use migration::{Alias, DynIden};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, Func, Query};
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, DatabaseConnection, DbErr, JoinType, QuerySelect, TransactionTrait, Unchanged};

pub async fn list_recipes(
    limit: u64,
    offset: u64,
    search: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    db: &DatabaseConnection,
) -> Result<Vec<entity::recipes::Model>, DbErr> {
    let mut query = entity::recipes::Entity::find().limit(limit).offset(offset);

    if let Some(search) = search {
        let mut cond = Condition::all();

        for s in search {
            cond = cond.add(
                Expr::expr(Func::lower(Expr::col((
                    entity::recipes::Entity,
                    entity::recipes::Column::Name,
                ))))
                .like(format!("%{}%", s.to_lowercase())),
            );
        }

        query = query.filter(cond);
    }

    if let Some(tags) = tags {
        let tags_search_tbl: DynIden = sea_orm::sea_query::SeaRc::new(Alias::new("tags_search"));

        query = query
            .join(JoinType::InnerJoin, entity::recipes::Relation::RecipesTags.def())
            .join(JoinType::InnerJoin, entity::recipes_tags::Relation::Tags.def())
            .filter(
                Condition::any()
                    .add(
                        Expr::col((entity::tags::Entity, entity::tags::Column::Name)).in_subquery(
                            Query::select()
                                .column(entity::tags::Column::Name)
                                .from_as(entity::tags::Entity, tags_search_tbl.clone())
                                .and_where(Expr::col((tags_search_tbl, entity::tags::Column::Name)).is_in(tags))
                                .to_owned(),
                        ),
                    )
                    .to_owned(),
            );
    }

    query.all(db).await
}

pub async fn count_recipes(
    search: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    db: &DatabaseConnection,
) -> Result<u64, DbErr> {
    let mut query = entity::recipes::Entity::find();

    if let Some(search) = search {
        let mut cond = Condition::all();

        for s in search {
            cond = cond.add(
                Expr::expr(Func::lower(Expr::col((
                    entity::recipes::Entity,
                    entity::recipes::Column::Name,
                ))))
                .like(format!("%{}%", s.to_lowercase())),
            );
        }

        query = query.filter(cond);
    }

    if let Some(tags) = tags {
        let tags_search_tbl: DynIden = sea_orm::sea_query::SeaRc::new(Alias::new("tags_search"));

        query = query
            .join(JoinType::InnerJoin, entity::recipes::Relation::RecipesTags.def())
            .join(JoinType::InnerJoin, entity::recipes_tags::Relation::Tags.def())
            .filter(
                Condition::any()
                    .add(
                        Expr::col((entity::tags::Entity, entity::tags::Column::Name)).in_subquery(
                            Query::select()
                                .column(entity::tags::Column::Name)
                                .from_as(entity::tags::Entity, tags_search_tbl.clone())
                                .and_where(Expr::col((tags_search_tbl, entity::tags::Column::Name)).is_in(tags))
                                .to_owned(),
                        ),
                    )
                    .to_owned(),
            );
    }

    query.count(db).await
}

pub async fn get_recipe_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<entity::recipes::Model>, DbErr> {
    entity::recipes::Entity::find_by_id(id).one(db).await
}

#[derive(SimpleObject, InputObject, Debug)]
pub struct RecipeInput {
    name: String,
    description: Option<String>,
    tags: Option<Vec<i64>>,
}

pub async fn create_recipe(
    recipe_values: RecipeInput,
    owner_id: i64,
    db: &DatabaseConnection,
) -> Result<entity::recipes::Model, DbErr> {
    let now = Utc::now().naive_utc();

    let new_recipe = entity::recipes::ActiveModel {
        name: Set(recipe_values.name),
        description: Set(recipe_values.description),
        inserted_at: Set(now),
        updated_at: Set(now),
        owner_id: Set(Some(owner_id)),
        ..Default::default()
    };

    db.transaction::<_, entity::recipes::Model, DbErr>(|txn| {
        Box::pin(async move {
            let recipe = new_recipe.insert(txn).await?;

            if let Some(tags) = recipe_values.tags {
                for tag in tags {
                    entity::recipes_tags::ActiveModel {
                        recipe_id: Set(recipe.id),
                        tag_id: Set(tag),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                }
            }

            Ok(recipe)
        })
    })
    .await
    .map_err(|_e| DbErr::Query(sea_orm::RuntimeErr::Internal("Transaction failed".to_string())))
}

pub async fn update_recipe(
    id: i64,
    recipe_input: RecipeInput,
    db: &DatabaseConnection,
) -> Result<entity::recipes::Model, DbErr> {
    let recipe = entity::recipes::ActiveModel {
        id: Unchanged(id),
        name: Set(recipe_input.name),
        description: Set(recipe_input.description),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    db.transaction::<_, entity::recipes::Model, DbErr>(|txn| {
        Box::pin(async move {
            let recipe = recipe.update(txn).await?;

            if let Some(tags) = recipe_input.tags {
                entity::recipes_tags::Entity::delete_many()
                    .filter(entity::recipes_tags::Column::TagId.is_not_in(tags.clone()))
                    .filter(entity::recipes_tags::Column::RecipeId.eq(recipe.id))
                    .exec(txn)
                    .await?;

                let existing_tag_ids = entity::recipes_tags::Entity::find()
                    .filter(entity::recipes_tags::Column::TagId.is_in(tags.clone()))
                    .filter(entity::recipes_tags::Column::RecipeId.eq(recipe.id))
                    .all(txn)
                    .await?;

                let new_tags = tags
                    .into_iter()
                    .filter(|tag| !existing_tag_ids.iter().any(|t| t.tag_id == *tag))
                    .collect::<Vec<i64>>();

                for tag in new_tags {
                    entity::recipes_tags::ActiveModel {
                        recipe_id: Set(recipe.id),
                        tag_id: Set(tag),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                }
            }

            Ok(recipe)
        })
    })
    .await
    .map_err(|_e| DbErr::Query(sea_orm::RuntimeErr::Internal("Transaction failed".to_string())))
}

pub async fn delete_recipe(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(entity::recipes::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1)
}
