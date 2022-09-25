use sea_orm_migration::prelude::*;

use crate::{
    m20220917_082200_create_ingredients::Ingredients, m20220917_082516_create_steps::Steps,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StepsIngridients::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StepsIngridients::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StepsIngridients::StepId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StepsIngridients::IngridientId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StepsIngridients::Amount).double().not_null())
                    .col(ColumnDef::new(StepsIngridients::Unit).string().not_null())
                    .col(
                        ColumnDef::new(StepsIngridients::Annotation)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StepsIngridients::InsertedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StepsIngridients::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("steps_ingridients_step_id_fkey")
                    .from(StepsIngridients::Table, StepsIngridients::StepId)
                    .to(Steps::Table, Steps::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("steps_ingridients_ingredient_id_fkey")
                    .from(StepsIngridients::Table, StepsIngridients::IngridientId)
                    .to(Ingredients::Table, Ingredients::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("steps_ingridients_step_id_idx")
                    .table(StepsIngridients::Table)
                    .col(StepsIngridients::StepId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("steps_ingridients_ingredient_id_idx")
                    .table(StepsIngridients::Table)
                    .col(StepsIngridients::IngridientId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StepsIngridients::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum StepsIngridients {
    Table,
    Id,
    StepId,
    IngridientId,
    Amount,
    Unit,
    Annotation,
    InsertedAt,
    UpdatedAt,
}
