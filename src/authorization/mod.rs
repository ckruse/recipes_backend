use sea_orm::DatabaseConnection;

use entity::users::Model as UserModel;

pub mod recipes_policy;

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
