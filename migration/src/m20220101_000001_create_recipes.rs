use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Recipes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Recipes::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Recipes::Name).string().not_null())
                    .col(ColumnDef::new(Recipes::Description).text())
                    .col(ColumnDef::new(Recipes::InsertedAt).timestamp().not_null())
                    .col(ColumnDef::new(Recipes::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Recipes::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Recipes {
    Table,
    Id,
    Name,
    Description,
    InsertedAt,
    UpdatedAt,
}
