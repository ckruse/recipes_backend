use async_graphql::Error;
use entity::users::{Model as UserModel, Role};
use sea_orm::DatabaseConnection;

pub mod ingredients_policy;
pub mod recipes_policy;
pub mod users_policy;

pub enum DefaultActions {
    List,
    Create,
    Get,
    Update,
    Delete,
}

pub trait Authorization<T, T1> {
    fn authorized(&self, action: T, user: Option<&UserModel>, resource: Option<&T1>, db: &DatabaseConnection) -> bool;
}

pub fn is_root(user: Option<&UserModel>) -> bool {
    if let Some(user) = user {
        if user.role == Role::Root {
            return true;
        }
    }

    false
}

pub fn authorized<T: Authorization<T1, T2>, T1, T2>(
    module: T,
    action: T1,
    user: Option<&UserModel>,
    resource: Option<&T2>,
    db: &DatabaseConnection,
) -> Result<(), Error> {
    if !module.authorized(action, user, resource, db) {
        return Err(Error::new("Unauthorized"));
    }

    Ok(())
}
