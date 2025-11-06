use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Result as RocketResult},
    serde::json::Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> RocketResult<'static> {
        let (status, msg) = match self {
            ApiError::NotFound => (Status::NotFound, "Not found".to_string()),
            ApiError::BadRequest(m) => (Status::BadRequest, m),
            ApiError::Db(e) => (Status::InternalServerError, e.to_string()),
            ApiError::Internal(m) => (Status::InternalServerError, m),
        };

        // Return `status` with a JSON error body.
        rocket::response::status::Custom(status, Json(ErrorBody { error: msg }))
            .respond_to(req)
    }
}

// (Optional) Handy alias you can use in handlers: ApiResult<T> = Result<Json<T>, ApiError>
pub type ApiResult<T> = Result<Json<T>, ApiError>;
