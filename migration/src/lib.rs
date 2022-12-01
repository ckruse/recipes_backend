pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20220916_134304_initial;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220916_134304_initial::Migration)]
    }
}
