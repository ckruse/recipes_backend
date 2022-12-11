use entity::ingredients::Model as IngredientModel;
use entity::users::{Model as UserModel, Role};
use sea_orm::DatabaseConnection;

use super::{Authorization, DefaultActions};

pub struct IngredientsPolicy;

impl Authorization<DefaultActions, IngredientModel> for IngredientsPolicy {
    fn authorized(
        &self,
        action: DefaultActions,
        user: Option<&UserModel>,
        _resource: Option<&IngredientModel>,
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
                }

                false
            }
            DefaultActions::Delete => {
                if let Some(user) = user {
                    if user.role == Role::Root {
                        return true;
                    }
                }

                false
            }
        }
    }
}
