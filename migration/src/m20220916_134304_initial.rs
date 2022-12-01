use sea_orm::Statement;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = include_str!("../initial.sql");
        let statements: Vec<&str> = sql.split("-- NEXT --").collect();

        for sql in statements {
            let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
            manager.get_connection().execute(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let statements = vec![
            "DROP TABLE steps_ingridients",
            "DROP TABLE steps",
            "DROP TABLE recipes_tags",
            "DROP TABLE recipes",
            "DROP TABLE tags",
            "DROP TABLE ingredient_units",
            "DROP TABLE ingridients",
            "DROP TABLE users",
        ];

        for sql in statements {
            let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
            manager.get_connection().execute(stmt).await?;
        }

        Ok(())
    }
}
