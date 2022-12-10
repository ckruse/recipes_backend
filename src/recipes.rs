use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::fs::File;
use std::io::copy;
use std::path::Path;

use async_graphql::*;
use chrono::Utc;
use image::imageops;
use image::GenericImageView;
use migration::{Alias, DynIden};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, Func, Query};
use sea_orm::ActiveValue::Set;
use sea_orm::{Condition, DatabaseConnection, DbErr, JoinType, QuerySelect, TransactionTrait, Unchanged};

use crate::utils::{correct_orientation, get_orientation, image_base_path, read_exif};

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
                Condition::any().add(
                    Expr::col((entity::tags::Entity, entity::tags::Column::Name)).in_subquery(
                        Query::select()
                            .column(entity::tags::Column::Name)
                            .from_as(entity::tags::Entity, tags_search_tbl.clone())
                            .and_where(Expr::col((tags_search_tbl, entity::tags::Column::Name)).is_in(tags))
                            .to_owned(),
                    ),
                ),
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
                Condition::any().add(
                    Expr::col((entity::tags::Entity, entity::tags::Column::Name)).in_subquery(
                        Query::select()
                            .column(entity::tags::Column::Name)
                            .from_as(entity::tags::Entity, tags_search_tbl.clone())
                            .and_where(Expr::col((tags_search_tbl, entity::tags::Column::Name)).is_in(tags))
                            .to_owned(),
                    ),
                ),
            );
    }

    query.count(db).await
}

pub async fn get_recipe_by_id(id: i64, db: &DatabaseConnection) -> Result<Option<entity::recipes::Model>, DbErr> {
    entity::recipes::Entity::find_by_id(id).one(db).await
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

#[derive(InputObject)]
pub struct RecipeInput {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<Upload>,
    pub tags: Option<Vec<i64>>,
}

pub async fn create_recipe(
    recipe_values: RecipeInput,
    picture: Option<UploadValue>,
    owner_id: i64,
    db: &DatabaseConnection,
) -> Result<entity::recipes::Model, DbErr> {
    let now = Utc::now().naive_utc();

    let mut new_recipe = entity::recipes::ActiveModel {
        name: Set(recipe_values.name),
        description: Set(recipe_values.description),
        inserted_at: Set(now),
        updated_at: Set(now),
        owner_id: Set(Some(owner_id)),
        ..Default::default()
    };

    if let Some(ref picture) = picture {
        new_recipe.image = Set(Some(picture.filename.clone()));
    }

    db.transaction::<_, entity::recipes::Model, DbErr>(|txn| {
        Box::pin(async move {
            let recipe = new_recipe.insert(txn).await?;

            if let Some(tags) = recipe_values.tags {
                for tag in tags {
                    entity::recipes_tags::ActiveModel {
                        recipe_id: Set(recipe.id),
                        tag_id: Set(tag),
                    }
                    .insert(txn)
                    .await?;
                }
            }

            if let Some(picture) = picture {
                save_picture(&recipe, picture)?;
            }

            Ok(recipe)
        })
    })
    .await
    .map_err(|_e| DbErr::Query(sea_orm::RuntimeErr::Internal("Transaction failed".to_string())))
}

pub async fn update_recipe(
    id: i64,
    values: RecipeInput,
    picture: Option<UploadValue>,
    db: &DatabaseConnection,
) -> Result<entity::recipes::Model, DbErr> {
    let mut recipe = entity::recipes::ActiveModel {
        id: Unchanged(id),
        name: Set(values.name),
        description: Set(values.description),
        updated_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    if let Some(ref picture) = picture {
        recipe.image = Set(Some(picture.filename.clone()));
    }

    db.transaction::<_, entity::recipes::Model, DbErr>(|txn| {
        Box::pin(async move {
            let recipe = recipe.update(txn).await?;

            if let Some(tags) = values.tags {
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
                    }
                    .insert(txn)
                    .await?;
                }
            }

            if let Some(picture) = picture {
                save_picture(&recipe, picture)?;
            }

            Ok(recipe)
        })
    })
    .await
    .map_err(|e| DbErr::Query(sea_orm::RuntimeErr::Internal(format!("Transaction failed: {}", e))))
}

fn save_picture(recipe: &entity::recipes::Model, mut picture: UploadValue) -> Result<(), DbErr> {
    let path = format!("{}/{}/", image_base_path(), recipe.id);

    std::fs::create_dir_all(path).map_err(|e| DbErr::Custom(format!("Failed to create picture directory: {}", e)))?;

    let path = match get_extension_from_filename(&picture.filename) {
        Some(ext) => format!("{}/{}/original.{}", image_base_path(), recipe.id, ext),
        None => format!("{}/{}/original.jpg", image_base_path(), recipe.id),
    };
    let mut file = File::create(path).map_err(|e| DbErr::Custom(format!("Failed to create picture: {}", e)))?;
    copy(&mut picture.content, &mut file).map_err(|e| DbErr::Custom(format!("Failed to copy picture: {}", e)))?;

    let recipe_ = recipe.clone();
    tokio::task::spawn_blocking(move || generate_pictures(recipe_));

    Ok(())
}

const THUMB_ASPEC_RATIO: f32 = 1.0;

fn generate_pictures(recipe: entity::recipes::Model) -> anyhow::Result<()> {
    let img = recipe.image.unwrap();
    let ext = get_extension_from_filename(&img).unwrap_or(".jpg");

    let path = format!("{}/{}/original.{}", image_base_path(), recipe.id, ext);
    let exif = read_exif(&path)?;
    let orientation = get_orientation(&exif);

    let mut img = image::open(path).expect("Failed to open image");
    img = correct_orientation(img, orientation);

    let path = format!("{}/{}/large.{}", image_base_path(), recipe.id, ext);
    let new_img = img.resize(800, 600, imageops::FilterType::CatmullRom);
    new_img.save(path)?;

    let path = format!("{}/{}/thumbnail.{}", image_base_path(), recipe.id, ext);
    let (width, height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;

    let img = if aspect_ratio != THUMB_ASPEC_RATIO {
        let mid_x = width / 2;
        let mid_y = height / 2;

        if width > height {
            img.crop(mid_x - (height / 2), mid_y - (height / 2), height, height)
        } else {
            img.crop(mid_x - (width / 2), mid_y - (width / 2), width, width)
        }
    } else {
        img
    };

    let new_img = img.resize_exact(600, 600, imageops::FilterType::CatmullRom);
    new_img.save(path)?;

    Ok(())
}

pub async fn delete_recipe(id: i64, db: &DatabaseConnection) -> Result<bool, DbErr> {
    let deleted = entity::recipes::Entity::delete_by_id(id).exec(db).await?.rows_affected == 1;

    if deleted {
        let path = format!("{}/{}", image_base_path(), id);
        let _ = remove_dir_all(path);
    }

    Ok(deleted)
}
