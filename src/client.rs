use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use domain::db::deezer::{AlbumInputDeezer, AuthorInputDeezer};
use domain::db::soundcloud::{AuthorInputSoundcloud, TrackInputSoundcloud};
use domain::db::user::{IsUserExistsRes, UserTable, UserWithPlaylists};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, pool};
use sqlx::spec_error::SpecErrorWrapper;
use sqlx::types::Json;
use uuid::Uuid;

use crate::errors::session::{SessionCreationError, SessionUpdateError};
use crate::errors::sqlx_error::SqlxErrorWrapper;
use crate::errors::user::UserCreationError;

type SqlxResult<T> = Result<T, SqlxErrorWrapper>;



#[derive(Debug, Clone)]
pub struct PostgresDb {
    pool: pool::Pool<Postgres>,
}

impl PostgresDb {
    pub async fn new(url: String) -> Self {
        Self {
            pool: PgPoolOptions::new().connect(&url).await.unwrap(),
        }
    }

    // --- SOUNDCLOUD ---
    pub async fn add_track_soundcloud(
        &self,
        track: &TrackInputSoundcloud,
        author: &AuthorInputSoundcloud,
    ) -> SqlxResult<()> {
        sqlx::query("CALL add_track_soundcloud($1, $2)")
            .bind(track)
            .bind(author)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn record_listening_soundcloud(&self, track_id: i64) -> SqlxResult<bool> {
        let result: bool = sqlx::query_scalar("SELECT record_listen_soundcloud($1)")
            .bind(track_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    // --- DEEZER ---
    pub async fn add_album_deezer(
        &self,
        author: &AuthorInputDeezer,
        album: &AlbumInputDeezer,
        tracks: &[TrackInputSoundcloud],
    ) -> SqlxResult<()> {
        sqlx::query("CALL add_album_deezer($1, $2, $3)")
            .bind(author)
            .bind(album)
            .bind(tracks)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn record_listening_deezer(&self, track_id: i64) -> Result<bool, sqlx::Error> {
        let result: bool = sqlx::query_scalar("SELECT record_listen_deezer($1)")
            .bind(track_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    // Auth

    // --- User ---

    pub async fn add_user(
        &self,
        username: String,
        email: String,
        password_hash: String,
    ) -> SqlxResult<Result<i64, UserCreationError>> {
        let result: i64 = sqlx::query_scalar("SELECT add_user($1, $2, $3)")
            .bind(email)
            .bind(username)
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await?;

        match result {
            -1 => Ok(Err(UserCreationError::EmailAlreadyExists)),
            ..0 => Ok(Err(UserCreationError::Other)),
            _ => Ok(Ok(result)),
        }
    }

    pub async fn get_user_by_id(&self, id: i64) -> SqlxResult<Option<UserTable>> {
        let user = sqlx::query_as("SELECT * FROM app_user WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool) // Assuming `self.pool` is the database connection pool
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: String) -> SqlxResult<Option<UserTable>> {
        let user = sqlx::query_as("SELECT * FROM app_user WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool) // Assuming `self.pool` is the database connection pool
            .await?;

        Ok(user)
    }

    pub async fn check_is_user_exists(&self, username: String, email: String) -> SqlxResult<IsUserExistsRes> {
        let is_exists: i16 = sqlx::query_scalar(
            "SELECT get_user_with_playlists_json($1, $2)"
        )
            .bind(email)
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        match is_exists {
            0 => Ok(IsUserExistsRes::NotExists),
            1 => Ok(IsUserExistsRes::EmailExists),
            2 => Ok(IsUserExistsRes::UsernameExists),
            3 => Ok(IsUserExistsRes::EmailAndUsernameExists),
            _ => panic!("Database return wrong data")
        }
    }

    pub async fn get_user_with_playlists(&self, id: i64) -> SqlxResult<Option<UserWithPlaylists>> {
        let user_record: Option<(Json<UserWithPlaylists>,)> = sqlx::query_as(
            "SELECT get_user_with_playlists_json($1)"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user_record.map(|(json_wrapper,)| json_wrapper.0))
    }

    pub async fn update_password_hash(&self, id: i64, password_hash: &str) -> sqlx::Result<()> {
        sqlx::query("UPDATE app_user SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- SESSIONS ---

    pub async fn add_session(
        &self,
        id: Uuid,
        user_id: &str, // Using &str to match VARCHAR(18) in your SQL function
        sn: Uuid,
        expires_at: DateTime<Utc>,
    ) -> SqlxResult<Option<SessionCreationError>> {
        // We use query_scalar to fetch the single return value (a SMALLINT) from the function.
        let result_code: i16 = sqlx::query_scalar("SELECT add_session($1, $2, $3, $4)")
            .bind(id)
            .bind(user_id)
            .bind(sn)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await?;

        match result_code {
            // A positive return value indicates success (it returns the new ID, which we ignore here)
            0 => Ok(None),
            -1 => Ok(Some(SessionCreationError::IdAlreadyExists)),
            -2 => Ok(Some(SessionCreationError::UserNotFound)), // Handle -10 and any other unexpected codes
            _ => panic!("UNEXPECTED RETURN VALUE"),
        }
    }

    pub async fn extend_session(
        &self,
        id: Uuid,
        new_sn: Uuid,
        expires_at: DateTime<Utc>,
        old_sn: Uuid,
    ) -> SqlxResult<Option<SessionUpdateError>> {
        let result_code: i16 = sqlx::query_scalar("SELECT extend_session($1, $2, $3, $4)")
            .bind(id)
            .bind(old_sn)
            .bind(new_sn)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await?;

        match result_code {
            0 => Ok(None),
            1 => Ok(Some(SessionUpdateError::NotFound)),
            2 => Ok(Some(SessionUpdateError::InvalidSerialNumber)),
            3 => Ok(Some(SessionUpdateError::Expired)),
            _ => panic!("UNEXPECTED RETURN VALUE"), // Handle any other unexpected codes
        }
    }
}
