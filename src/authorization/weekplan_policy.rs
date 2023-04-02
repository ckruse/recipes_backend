use entity::users::Model as UserModel;
use entity::weekplans::Model as WeekplanModel;
use sea_orm::DatabaseConnection;

use super::{Authorization, DefaultActions};

pub struct WeekplanPolicy;

impl Authorization<DefaultActions, WeekplanModel> for WeekplanPolicy {
    fn authorized(
        &self,
        action: DefaultActions,
        user: Option<&UserModel>,
        resource: Option<&WeekplanModel>,
        _db: &DatabaseConnection,
    ) -> bool {
        match action {
            DefaultActions::List => user.is_some(),
            DefaultActions::Create => user.is_some(),
            DefaultActions::Get => user.is_some() && resource.is_some(),
            DefaultActions::Update => user.is_some() && resource.is_some(),
            DefaultActions::Delete => user.is_some() && resource.is_some(),
        }
    }
}
