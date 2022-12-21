use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "fitting")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub recipe_id: i64,
    #[sea_orm(primary_key)]
    pub fitting_recipe_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::recipes::Entity",
        from = "Column::RecipeId",
        to = "super::recipes::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Recipes,
    #[sea_orm(
        belongs_to = "super::recipes::Entity",
        from = "Column::FittingRecipeId",
        to = "super::recipes::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    FittingRecipes,
}

impl Related<super::recipes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Recipes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
