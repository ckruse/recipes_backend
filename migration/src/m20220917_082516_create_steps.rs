use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_recipes::Recipes;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Steps::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Steps::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Steps::RecipeId).big_integer().not_null())
                    .col(ColumnDef::new(Steps::Position).integer().not_null())
                    .col(ColumnDef::new(Steps::Description).text())
                    .col(ColumnDef::new(Steps::InsertedAt).timestamp().not_null())
                    .col(ColumnDef::new(Steps::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("steps_recipe_id_fkey")
                    .from(Steps::Table, Steps::RecipeId)
                    .to(Recipes::Table, Recipes::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("steps_recipe_id_idx")
                    .table(Steps::Table)
                    .col(Steps::RecipeId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Steps::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Steps {
    Table,
    Id,
    RecipeId,
    Position,
    Description,
    InsertedAt,
    UpdatedAt,
}
