use entity::recipes::Model as RecipeModel;
use entity::users::{Model as UserModel, Role};
use sea_orm::DatabaseConnection;

use super::{Authorization, DefaultActions};

pub struct RecipesPolicy;

impl Authorization<DefaultActions, RecipeModel> for RecipesPolicy {
    fn authorized(
        &self,
        action: DefaultActions,
        user: Option<&UserModel>,
        resource: Option<&RecipeModel>,
        _db: &DatabaseConnection,
    ) -> bool {
        match action {
            DefaultActions::List => true,
            DefaultActions::Create => user.is_some(),
            DefaultActions::Get => true,
            DefaultActions::Update => {
                if let Some(user) = user {
                    if user.role == Role::Root {
                        return true;
                    }

                    if let Some(recipe) = resource {
                        if recipe.owner_id == Some(user.id) {
                            return true;
                        }
                    }
                }

                false
            }
            DefaultActions::Delete => {
                if let Some(user) = user {
                    if user.role == Role::Root {
                        return true;
                    }

                    if let Some(recipe) = resource {
                        if recipe.owner_id == Some(user.id) {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }
}
