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
    async fn create_weekplan(
        &self,
        ctx: &Context<'_>,
        week: NaiveDate,
        tags: Vec<String>,
        portions: Option<i32>,
        days: Option<Vec<u32>>,
    ) -> Result<Vec<Weekplan>> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        authorized(WeekplanPolicy, DefaultActions::Create, user, None, db)?;

        // due to policy check user is always Some
        let user = user.unwrap();

        crate::weekplan::create_weekplan_for_week(week, user.to_owned(), tags, portions.unwrap_or(2), days, db)
            .await
            .map_err(|e| e.into())
    }

    async fn replace_weekplan_recipe(&self, ctx: &Context<'_>, id: i64, tags: Vec<String>) -> Result<Weekplan> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let weekplan = crate::weekplan::get_weekplan_by_id(id, db).await?;
        authorized(WeekplanPolicy, DefaultActions::Update, user, weekplan.as_ref(), db)?;

        // due to policy check the entry is a Some
        let weekplan = weekplan.unwrap();

        crate::weekplan::replace_weekplan_recipe(weekplan, tags, db)
            .await
            .map_err(|e| e.into())
    }

    async fn replace_weekplan_recipe_with_recipe(
        &self,
        ctx: &Context<'_>,
        id: i64,
        recipe_id: i64,
    ) -> Result<Weekplan> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let weekplan = crate::weekplan::get_weekplan_by_id(id, db).await?;
        authorized(WeekplanPolicy, DefaultActions::Update, user, weekplan.as_ref(), db)?;

        // due to policy check the entry is a Some
        let weekplan = weekplan.unwrap();

        crate::weekplan::replace_weekplan_recipe_with_recipe(weekplan, recipe_id, db)
            .await
            .map_err(|e| e.into())
    }

    async fn delete_weekplan(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        let user = ctx.data_opt::<entity::users::Model>();
        let db = ctx.data::<DatabaseConnection>()?;

        let weekplan = crate::weekplan::get_weekplan_by_id(id, db).await?;
        authorized(WeekplanPolicy, DefaultActions::Delete, user, weekplan.as_ref(), db)?;

        crate::weekplan::delete_weekplan(id, db).await.map_err(|e| e.into())
    }
}
