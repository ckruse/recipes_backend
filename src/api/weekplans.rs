use async_graphql::*;
use chrono::NaiveDate;
use entity::weekplans::Model as Weekplan;
use sea_orm::DatabaseConnection;

use crate::authorization::weekplan_policy::WeekplanPolicy;
use crate::authorization::{authorized, DefaultActions};

#[derive(Default)]
pub struct WeekplansQueries;

#[derive(Default)]
pub struct WeekplansMutations;

#[Object]
impl WeekplansQueries {
    async fn weekplans(&self, ctx: &Context<'_>, week: NaiveDate) -> Result<Vec<Weekplan>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(WeekplanPolicy, DefaultActions::List, user, None, db)?;

        // due to policy check user is always Some
        let user = user.unwrap();

        crate::weekplan::list_weekplan(&week, user, db)
            .await
            .map_err(|e| e.into())
    }
}

#[Object]
impl WeekplansMutations {
    async fn create_weekplan(&self, ctx: &Context<'_>, week: NaiveDate, tags: Vec<String>) -> Result<Vec<Weekplan>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(WeekplanPolicy, DefaultActions::Create, user, None, db)?;

        // due to policy check user is always Some
        let user = user.unwrap();

        crate::weekplan::create_weekplan_for_week(week, user.to_owned(), tags, db)
            .await
            .map_err(|e| e.into())
    }
}
