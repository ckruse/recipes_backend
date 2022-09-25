use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use sea_orm::entity::prelude::*;
use sea_orm::DatabaseConnection;

use entity::users;

use crate::token::create_jwt;

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
