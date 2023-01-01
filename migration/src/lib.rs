pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20220916_134304_initial;
mod m20221211_100047_make_amount_optional;
mod m20221218_173259_make_tagname_unique;
mod m20221219_070643_add_fits_to;
mod m20221221_135800_add_name_to_steps;
mod m20221221_145615_add_default_quantity_to_recipes;
mod m20230101_124914_make_tag_name_not_null;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220916_134304_initial::Migration),
            Box::new(m20221211_100047_make_amount_optional::Migration),
            Box::new(m20221218_173259_make_tagname_unique::Migration),
            Box::new(m20221219_070643_add_fits_to::Migration),
            Box::new(m20221221_135800_add_name_to_steps::Migration),
            Box::new(m20221221_145615_add_default_quantity_to_recipes::Migration),
            Box::new(m20230101_124914_make_tag_name_not_null::Migration),
        ]
    }
}
