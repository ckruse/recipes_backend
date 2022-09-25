pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_recipes;
mod m20220916_135604_create_tags;
mod m20220917_080834_create_recipes_tags;
mod m20220917_082200_create_ingredients;
mod m20220917_082516_create_steps;
mod m20220917_083028_create_steps_ingridients;
mod m20220917_085315_create_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_recipes::Migration),
            Box::new(m20220916_135604_create_tags::Migration),
            Box::new(m20220917_080834_create_recipes_tags::Migration),
            Box::new(m20220917_082200_create_ingredients::Migration),
            Box::new(m20220917_082516_create_steps::Migration),
            Box::new(m20220917_083028_create_steps_ingridients::Migration),
            Box::new(m20220917_085315_create_users::Migration),
        ]
    }
}
