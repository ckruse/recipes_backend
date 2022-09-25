use sea_orm_migration::prelude::*;

use crate::{m20220916_135004_create_recipes::Recipes, m20220916_135604_create_tags::Tags};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RecipesTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RecipesTags::RecipeId).big_integer().not_null())
                    .col(ColumnDef::new(RecipesTags::TagId).big_integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("recipes_tags_recipe_id_fkey")
                    .from(RecipesTags::Table, RecipesTags::RecipeId)
                    .to(Recipes::Table, Recipes::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("recipes_tags_tag_id_fkey")
                    .from(RecipesTags::Table, RecipesTags::RecipeId)
                    .to(Tags::Table, Tags::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("recipes_tags_recipe_id_idx")
                    .table(RecipesTags::Table)
                    .col(RecipesTags::RecipeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("recipes_tags_tag_id_idx")
                    .table(RecipesTags::Table)
                    .col(RecipesTags::TagId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RecipesTags::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum RecipesTags {
    Table,
    RecipeId,
    TagId,
}
