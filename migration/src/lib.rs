pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20220916_134304_initial;
mod m20221211_100047_make_amount_optional;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220916_134304_initial::Migration),
            Box::new(m20221211_100047_make_amount_optional::Migration),
        ]
    }
}
