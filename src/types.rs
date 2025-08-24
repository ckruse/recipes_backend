use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use jwt_simple::prelude::*;
use sea_orm::{DatabaseConnection, DbErr};

#[derive(Clone, Debug)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub token_key: HS512Key,
}

#[derive(Debug, serde::Serialize)]
pub struct HttpError {
    pub message: String,
    pub is_error: bool,
    pub code: u16,
}

impl HttpError {
    pub fn not_found(msg: Option<impl Into<String>>) -> HttpError {
        HttpError {
            message: msg.map_or_else(|| "not found".to_owned(), |m| m.into()),
            is_error: true,
            code: 404,
        }
    }
}

impl From<DbErr> for HttpError {
    fn from(err: DbErr) -> Self {
        Self {
            message: err.to_string(),
            is_error: true,
            code: 500,
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "status": "error", "code": self.code, "message": self.message });
        (StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(body)).into_response()
    }
}
