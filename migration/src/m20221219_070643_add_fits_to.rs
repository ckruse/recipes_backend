use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fitting::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Fitting::RecipeId).big_integer().not_null())
                    .col(ColumnDef::new(Fitting::FittingRecipeId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Fitting::Table, Fitting::RecipeId)
                            .to(Recipes::Table, Recipes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Fitting::Table, Fitting::FittingRecipeId)
                            .to(Recipes::Table, Recipes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .table(Fitting::Table)
                            .col(Fitting::RecipeId)
                            .col(Fitting::FittingRecipeId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Fitting::Table).to_owned()).await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Fitting {
    Table,
    RecipeId,
    FittingRecipeId,
}

#[derive(Iden)]
enum Recipes {
    Table,
    Id,
}
