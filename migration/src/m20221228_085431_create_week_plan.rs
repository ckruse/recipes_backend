use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Weekplans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Weekplans::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Weekplans::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Weekplans::Date).date().not_null())
                    .col(ColumnDef::new(Weekplans::RecipeId).big_integer().not_null())
                    .col(ColumnDef::new(Weekplans::Portions).integer().not_null())
                    .col(ColumnDef::new(Weekplans::InsertedAt).timestamp().not_null())
                    .col(ColumnDef::new(Weekplans::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Weekplans::Table, Weekplans::RecipeId)
                            .to(Recipes::Table, Recipes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Weekplans::Table, Weekplans::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Weekplans::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Weekplans {
    Table,
    Id,
    UserId,
    Date,
    RecipeId,
    Portions,
    InsertedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Recipes {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
