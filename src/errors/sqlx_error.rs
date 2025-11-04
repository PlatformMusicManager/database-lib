use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub enum SqlxErrorWrapper {
    SqlxError(sqlx::Error),
}

// This allows us to use the `?` operator to automatically convert
impl From<sqlx::Error> for SqlxErrorWrapper {
    fn from(err: sqlx::Error) -> Self {
        SqlxErrorWrapper::SqlxError(err)
    }
}

impl IntoResponse for SqlxErrorWrapper {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SqlxErrorWrapper::SqlxError(sqlx_error) => {
                // Now we can match on the specific sqlx::Error variants
                match sqlx_error {
                    sqlx::Error::RowNotFound => {
                        (StatusCode::NOT_FOUND, "The requested resource was not found.".to_string())
                    }
                    // You can add more specific sqlx error matches here
                    sqlx::Error::Database(err) if err.is_unique_violation() => {
                        (StatusCode::CONFLICT, "A record with this value already exists.".to_string())
                    }
                    _ => {
                        // For any other database error, return a generic 500
                        (StatusCode::INTERNAL_SERVER_ERROR, "An internal database error occurred.".to_string())
                    }
                }
            }
        };

        // Build a JSON response
        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}