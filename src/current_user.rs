use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use entity::users;
use jwt_simple::prelude::*;
use sea_orm::DatabaseConnection;

use crate::types::{AppState, HttpError};

pub(crate) async fn current_user(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = get_token_from_header_or_cookie(&headers, &jar);

    let user = get_user_from_str_token(&state.token_key, token, &state.conn)
        .await
        .map_err(|e| {
            StatusCode::from_u16(e.code)
                .ok()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

fn get_token_from_header_or_cookie(headers: &HeaderMap, jar: &CookieJar) -> Option<String> {
    let token = headers
        .get("Authorization")
        .map(|v| v.to_str().unwrap_or_default())
        .map(|t| t.trim_start_matches("Bearer "))
        .unwrap_or_default();

    if token.is_empty() {
        jar.get("recipes_auth").map(|v| v.value().to_string())
    } else {
        Some(token.to_string())
    }
}

pub async fn get_user_from_str_token(
    key: &HS512Key,
    token: Option<String>,
    db: &DatabaseConnection,
) -> Result<Option<users::Model>, HttpError> {
    let Some(token) = token else {
        return Ok(None);
    };

    let Some(claims) = key.verify_token::<NoCustomClaims>(&token, None).ok() else {
        return Ok(None);
    };

    let Some(user_id) = claims.subject.unwrap_or_default().parse::<i64>().ok() else {
        return Ok(None);
    };

    Ok(crate::users::get_user_by_id(user_id, db).await)
}
