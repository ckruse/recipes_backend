use entity::users::{Model as UserModel, Role};
use sea_orm::DatabaseConnection;

use super::{Authorization, DefaultActions};

pub struct UsersPolicy;

impl Authorization<DefaultActions, UserModel> for UsersPolicy {
    fn authorized(
        &self,
        action: DefaultActions,
        user: Option<&UserModel>,
        resource: Option<&UserModel>,
        _db: &DatabaseConnection,
    ) -> bool {
        match action {
            DefaultActions::List => true,

            DefaultActions::Create => true,

            DefaultActions::Get => true,

            DefaultActions::Update => {
                if let Some(user) = user {
                    if user.role == Role::Root {
                        return true;
                    }

                    if let Some(other_user) = resource {
                        if other_user.id == user.id {
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

                    if let Some(other_user) = resource {
                        if other_user.id == user.id {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }
}
