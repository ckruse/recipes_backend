use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString};
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;

use async_graphql::*;

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{DatabaseConnection, Order, QueryOrder, QuerySelect};

use entity::users::{self, Role};

use crate::token::create_jwt;

#[derive(SimpleObject, InputObject)]
pub struct UserInput {
    pub email: String,
    pub password: Option<String>,
    pub name: Option<String>,
    pub role: Role,
}

pub async fn get_user_by_email(email: String, db: &DatabaseConnection) -> Option<users::Model> {
    match users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await
    {
        Ok(user) => user,
        _ => None,
    }
}

pub async fn get_user_by_id(id: i64, db: &DatabaseConnection) -> Option<users::Model> {
    match users::Entity::find_by_id(id).one(db).await {
        Ok(user) => user,
        _ => None,
    }
}

pub async fn authenticate_user(
    email: String,
    password: String,
    db: &DatabaseConnection,
) -> Option<(String, users::Model)> {
    match get_user_by_email(email, db).await {
        Some(user) => {
            if verify_password(&user.encrypted_password, &password) {
                Some((create_jwt(&user).ok()?, user))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn verify_password(hash: &Option<String>, password: &str) -> bool {
    if hash.is_none() {
        return false;
    }

    match PasswordHash::new(&hash.clone().unwrap()) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        _ => false,
    }
}

pub async fn list_users(
    limit: u64,
    offset: u64,
    search: Option<String>,
    db: &DatabaseConnection,
) -> Result<Vec<users::Model>, DbErr> {
    let mut query = users::Entity::find();

    if let Some(search) = search {
        query = query.filter(users::Column::Email.like(&format!("%{}%", search)));
    }

    query
        .order_by(users::Column::Email, Order::Asc)
        .limit(limit)
        .offset(offset)
        .all(db)
        .await
}

pub async fn count_users(search: Option<String>, db: &DatabaseConnection) -> Result<usize, DbErr> {
    let mut query = users::Entity::find();

    if let Some(search) = search {
        query = query.filter(users::Column::Email.like(&format!("%{}%", search)));
    }

    query.count(db).await
}

pub async fn get_user(id: i64, db: &DatabaseConnection) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(id).one(db).await
}

pub async fn create_user(user_values: UserInput, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let password_hash = if let Some(password) = user_values.password {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .ok()
            .and_then(|v| Some(v.to_string()))
    } else {
        None
    };

    users::ActiveModel {
        active: Set(true),
        email: Set(user_values.email),
        encrypted_password: Set(password_hash),
        name: Set(user_values.name),
        role: Set(user_values.role),
        inserted_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn update_user(id: i64, user_values: UserInput, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let password_hash = if let Some(password) = user_values.password {
        let salt = SaltString::generate(&mut OsRng);
        let value = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .and_then(|v| Ok(v.to_string()))
            .ok();

        Set(value)
    } else {
        Unchanged(None)
    };

    users::ActiveModel {
        id: Unchanged(id),
        active: Set(true),
        email: Set(user_values.email),
        encrypted_password: password_hash,
        name: Set(user_values.name),
        role: Set(user_values.role),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await
}

pub async fn delete_user(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(entity::users::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1)
}
