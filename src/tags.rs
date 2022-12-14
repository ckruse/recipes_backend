use async_graphql::*;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{DatabaseConnection, DbErr, QueryOrder, QuerySelect};

pub async fn list_tags(
    limit: Option<u64>,
    offset: Option<u64>,
    search: Option<String>,
    db: &DatabaseConnection,
) -> Result<Vec<entity::tags::Model>, DbErr> {
    let mut q = entity::tags::Entity::find();

    if let Some(limit) = limit {
        q = q.limit(limit);
    }

    if let Some(offset) = offset {
        q = q.offset(offset);
    }

    if let Some(search) = search {
        q = q.filter(entity::tags::Column::Name.like(&format!("%{}%", search)));
    }

    q.order_by_asc(entity::tags::Column::Name).all(db).await
}

pub async fn count_tags(search: Option<String>, db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut q = entity::tags::Entity::find();

    if let Some(search) = search {
        q = q.filter(entity::tags::Column::Name.like(&format!("%{}%", search)));
    }

    q.count(db).await
}

pub async fn get_tag_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<entity::tags::Model>, DbErr> {
    entity::tags::Entity::find_by_id(id).one(db).await
}

pub async fn create_tag(name: String, db: &DatabaseConnection) -> Result<entity::tags::Model, DbErr> {
    let now = Utc::now().naive_utc();

    let tag = entity::tags::ActiveModel {
        name: Set(name.to_lowercase()),
        inserted_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    tag.insert(db).await
}

pub async fn update_tag(id: i64, name: String, db: &DatabaseConnection) -> Result<entity::tags::Model, DbErr> {
    let tag: entity::tags::ActiveModel = entity::tags::ActiveModel {
        id: Unchanged(id),
        name: Set(name.to_lowercase()),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    tag.update(db).await
}

pub async fn delete_tag(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(entity::tags::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1)
}
