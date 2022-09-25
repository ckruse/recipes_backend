use anyhow::{anyhow, Result};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::users;

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(user: &entity::users::Model) -> Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))
        .map_err(|_| anyhow!("Error creating JWT"))
}

pub async fn decode_jwt(token: &str, db: &DatabaseConnection) -> Result<entity::users::Model> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map(|data| data.claims)
    .map_err(|_| anyhow!("Error decoding JWT"))?;

    let id = claims.sub.parse::<i64>()?;
    users::get_user_by_id(id, db).await.ok_or(anyhow!("User not found"))
}
