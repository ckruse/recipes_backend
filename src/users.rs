use std::fs::File;
use std::io::copy;

use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString};
use argon2::{Argon2, PasswordHasher};
use async_graphql::*;
use entity::users::{self, Role};
use image::imageops;
use rand_core::OsRng;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{DatabaseConnection, Order, QueryOrder, QuerySelect, TransactionTrait};

use crate::utils::{avatar_base_path, correct_orientation, get_extension_from_filename, get_orientation, read_exif};

#[derive(InputObject)]
pub struct UserInput {
    pub email: String,
    pub password: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<Upload>,
    pub role: Role,
    pub active: bool,
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

pub async fn authenticate_user(email: String, password: String, db: &DatabaseConnection) -> Option<users::Model> {
    match get_user_by_email(email, db).await {
        Some(user) => {
            if verify_password(&user.encrypted_password, &password) {
                Some(user)
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

pub async fn count_users(search: Option<String>, db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut query = users::Entity::find();

    if let Some(search) = search {
        query = query.filter(users::Column::Email.like(&format!("%{}%", search)));
    }

    query.count(db).await
}

pub async fn get_user(id: i64, db: &DatabaseConnection) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(id).one(db).await
}

pub async fn create_user(
    user_values: UserInput,
    avatar: Option<UploadValue>,
    db: &DatabaseConnection,
) -> Result<users::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let password_hash = if let Some(password) = user_values.password {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .ok()
            .map(|v| v.to_string())
    } else {
        None
    };

    db.transaction::<_, entity::users::Model, DbErr>(|txn| {
        Box::pin(async move {
            let user = users::ActiveModel {
                active: Set(true),
                email: Set(user_values.email),
                encrypted_password: Set(password_hash),
                name: Set(user_values.name),
                role: Set(user_values.role),
                inserted_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(txn)
            .await?;

            if let Some(avatar) = avatar {
                save_avatar(&user, avatar)?;
            }

            Ok(user)
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

pub async fn update_user(
    id: i64,
    user_values: UserInput,
    avatar: Option<UploadValue>,
    db: &DatabaseConnection,
) -> Result<users::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let password_hash = if let Some(password) = user_values.password {
        let salt = SaltString::generate(&mut OsRng);
        let value = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|v| v.to_string())
            .ok();

        Set(value)
    } else {
        Unchanged(None)
    };

    let mut user = users::ActiveModel {
        id: Unchanged(id),
        active: Set(true),
        email: Set(user_values.email),
        encrypted_password: password_hash,
        name: Set(user_values.name),
        role: Set(user_values.role),
        updated_at: Set(now),
        ..Default::default()
    };

    if let Some(ref picture) = avatar {
        user.avatar = Set(Some(picture.filename.clone()));
    }

    db.transaction::<_, entity::users::Model, DbErr>(|txn| {
        Box::pin(async move {
            let user = user.update(txn).await?;

            if let Some(avatar) = avatar {
                save_avatar(&user, avatar)?;
            }

            Ok(user)
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

fn save_avatar(user: &entity::users::Model, mut picture: UploadValue) -> Result<(), DbErr> {
    let path = format!("{}/{}/", avatar_base_path(), user.id);

    std::fs::create_dir_all(path).map_err(|e| DbErr::Custom(format!("Failed to create picture directory: {}", e)))?;

    let path = match get_extension_from_filename(&picture.filename) {
        Some(ext) => format!("{}/{}/original.{}", avatar_base_path(), user.id, ext),
        None => format!("{}/{}/original.jpg", avatar_base_path(), user.id),
    };
    let mut file = File::create(path).map_err(|e| DbErr::Custom(format!("Failed to create picture: {}", e)))?;
    copy(&mut picture.content, &mut file).map_err(|e| DbErr::Custom(format!("Failed to copy picture: {}", e)))?;

    let user_ = user.clone();
    tokio::task::spawn_blocking(move || generate_avatars(user_));

    Ok(())
}

fn generate_avatars(user: entity::users::Model) -> anyhow::Result<()> {
    let img = user.avatar.unwrap();
    let ext = get_extension_from_filename(&img).unwrap_or(".jpg");

    let path = format!("{}/{}/original.{}", avatar_base_path(), user.id, ext);
    let exif = read_exif(&path)?;
    let orientation = get_orientation(&exif);

    let mut img = image::open(path).expect("Failed to open image");
    img = correct_orientation(img, orientation);

    let path = format!("{}/{}/thumbnail.{}", avatar_base_path(), user.id, ext);
    let new_img = img.resize(150, 150, imageops::FilterType::CatmullRom);
    new_img.save(path).expect("Failed to save thumbnail");

    Ok(())
}

pub async fn delete_user(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(entity::users::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1)
}
