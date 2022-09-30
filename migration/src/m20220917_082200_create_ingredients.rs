use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ingredients::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ingredients::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ingredients::Name).string().not_null())
                    .col(ColumnDef::new(Ingredients::Reference).string().not_null())
                    .col(ColumnDef::new(Ingredients::Carbs).float().not_null())
                    .col(ColumnDef::new(Ingredients::Fat).float().not_null())
                    .col(ColumnDef::new(Ingredients::Proteins).float().not_null())
                    .col(ColumnDef::new(Ingredients::Alc).float().not_null())
                    .col(ColumnDef::new(Ingredients::InsertedAt).timestamp().not_null())
                    .col(ColumnDef::new(Ingredients::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ingredients::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Ingredients {
    Table,
    Id,
    Name,
    Reference,
    Carbs,
    Fat,
    Proteins,
    Alc,
    InsertedAt,
    UpdatedAt,
}
