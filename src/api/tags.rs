use async_graphql::*;
use sea_orm::DatabaseConnection;

#[derive(Default)]
pub struct TagsQueries;

#[derive(Default)]
pub struct TagsMutations;

#[Object]
impl TagsQueries {
    async fn tags(
        &self,
        ctx: &Context<'_>,
        limit: u64,
        offset: u64,
        search: Option<String>,
    ) -> Result<Vec<entity::tags::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::list_tags(limit, offset, search, db)
            .await
            .map_err(|e| e.into())
    }

    async fn count_tags(&self, ctx: &Context<'_>, search: Option<String>) -> Result<u64> {
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::count_tags(search, db).await.map_err(|e| e.into())
    }

    async fn tag(&self, ctx: &Context<'_>, id: i64) -> Result<Option<entity::tags::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::get_tag_by_id(id, db).await.map_err(|e| e.into())
    }
}

#[Object]
impl TagsMutations {
    async fn create_tag(&self, ctx: &Context<'_>, name: String) -> Result<entity::tags::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::create_tag(name, db).await.map_err(|e| e.into())
    }

    async fn update_tag(&self, ctx: &Context<'_>, id: i64, name: String) -> Result<entity::tags::Model> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::update_tag(id, name, db).await.map_err(|e| e.into())
    }

    async fn delete_tag(&self, ctx: &Context<'_>, id: i64) -> Result<bool> {
        ctx.data::<entity::users::Model>().map_err(|_| "Not logged in")?;
        let db = ctx.data::<DatabaseConnection>()?;
        crate::tags::delete_tag(id, db).await.map_err(|e| e.into())
    }
}
