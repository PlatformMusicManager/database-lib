use chrono::{Duration, Utc};
use domain::errors::db::session::{SessionCreationError, SessionUpdateError};
use domain::errors::db::sqlx_error::SqlxErrorWrapper;
use domain::errors::db::user::UserCreationError;
use domain::models::db::deezer::{
    AlbumInputDeezer, AuthorInputDeezer, FullAlbumResponse, TrackInputDeezer,
};
use domain::models::db::soundcloud::{
    AuthorInputSoundcloud, FullPlaylistResponse, FullTrackResponse, FullTracksResponse,
    PlaylistInputSoundcloud, TrackInputSoundcloud,
};
use domain::models::db::user::{IsUserExistsRes, UserTable, UserWithPlaylists};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::{Postgres, pool};
use std::collections::HashSet;
use std::ops::Add;
use uuid::Uuid;

type SqlxResult<T> = Result<T, SqlxErrorWrapper>;

#[derive(Debug, Clone)]
pub struct PostgresDb {
    pool: pool::Pool<Postgres>,
    refresh_token_ttl: Duration,
}

impl PostgresDb {
    pub async fn new(url: String, refresh_token_ttl: Duration) -> Self {
        Self {
            pool: PgPoolOptions::new().connect(&url).await.unwrap(),
            refresh_token_ttl,
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

    pub async fn get_track_full_soundcloud(
        &self,
        id: i64,
    ) -> SqlxResult<Option<FullTrackResponse>> {
        // We use fetch_one because the function ALWAYS returns a row (either data or NULL)
        // We cast the result to Option<Json<T>> to handle the SQL NULL safely
        let track: Option<Json<FullTrackResponse>> =
            sqlx::query_scalar("SELECT get_track_full_data_soundcloud($1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(track.map(|t| t.0))
    }

    pub async fn get_tracks_full_soundcloud(
        &self,
        track_ids: &[i64],
    ) -> SqlxResult<FullTracksResponse> {
        let result: Json<Vec<FullTrackResponse>> =
            sqlx::query_scalar("SELECT get_tracks_soundcloud_json($1)")
                .bind(track_ids)
                .fetch_one(&self.pool)
                .await?;

        let found_tracks = result.0;

        // Create a Set of found IDs for fast lookup
        let found_ids: HashSet<i64> = found_tracks.iter().map(|t| t.id).collect();

        // Filter the input list
        let not_found: Vec<i64> = track_ids
            .iter()
            .filter(|id| !found_ids.contains(id)) // Much faster
            .cloned()
            .collect();

        Ok(FullTracksResponse {
            not_found,
            found: found_tracks,
        })
    }

    pub async fn replace_or_create_playlist_soundcloud(
        &self,
        playlist: &PlaylistInputSoundcloud,
        playlist_author: &AuthorInputSoundcloud,
        tracks: &[TrackInputSoundcloud],
        track_authors: &[AuthorInputSoundcloud],
    ) -> SqlxResult<()> {
        sqlx::query("CALL replace_or_create_playlist_soundcloud($1, $2, $3, $4)")
            .bind(playlist)
            .bind(playlist_author)
            .bind(tracks)
            .bind(track_authors)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_playlist_soundcloud(
        &self,
        id: i64,
    ) -> SqlxResult<Option<FullPlaylistResponse>> {
        let res: Option<Json<FullPlaylistResponse>> =
            sqlx::query_scalar("SELECT get_playlist_with_tracks_soundcloud($1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(res.map(|el| el.0))
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
        tracks: &[TrackInputDeezer],
    ) -> SqlxResult<()> {
        sqlx::query("CALL add_album_deezer($1, $2, $3)")
            .bind(author)
            .bind(album)
            .bind(tracks)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_full_album(&self, album_id: i64) -> SqlxResult<Option<FullAlbumResponse>> {
        // We expect a single column containing JSON
        let result: Option<(Json<FullAlbumResponse>,)> =
            sqlx::query_as("SELECT get_album_details_json($1)")
                .bind(album_id)
                .fetch_optional(&self.pool)
                .await?;

        // Unwrap the Sqlx Json wrapper to get your struct
        Ok(result.map(|r| r.0.0))
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

    pub async fn check_is_user_exists(
        &self,
        username: &str,
        email: &str,
    ) -> SqlxResult<IsUserExistsRes> {
        let is_exists: i16 = sqlx::query_scalar("SELECT check_is_user_exists($1, $2)")
            .bind(email)
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        match is_exists {
            0 => Ok(IsUserExistsRes::NotExists),
            1 => Ok(IsUserExistsRes::EmailExists),
            2 => Ok(IsUserExistsRes::UsernameExists),
            3 => Ok(IsUserExistsRes::EmailAndUsernameExists),
            _ => panic!("Database return wrong data"),
        }
    }

    pub async fn get_user_with_playlists(&self, id: i64) -> SqlxResult<Option<UserWithPlaylists>> {
        let user_record: Option<(Json<UserWithPlaylists>,)> =
            sqlx::query_as("SELECT get_user_with_playlists_json($1)")
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
        user_id: i64, // Using &str to match VARCHAR(18) in your SQL function
        sn: Uuid,
    ) -> SqlxResult<Result<(), SessionCreationError>> {
        let expires_at = Utc::now().add(self.refresh_token_ttl);
        let result_code: i16 = sqlx::query_scalar("SELECT add_session($1, $2, $3, $4)")
            .bind(id)
            .bind(user_id)
            .bind(sn)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await?;

        match result_code {
            // A positive return value indicates success (it returns the new ID, which we ignore here)
            0 => Ok(Ok(())),
            -1 => Ok(Err(SessionCreationError::IdAlreadyExists)),
            -2 => Ok(Err(SessionCreationError::UserNotFound)), // Handle -10 and any other unexpected codes
            _ => panic!("UNEXPECTED RETURN VALUE"),
        }
    }

    pub async fn extend_session(
        &self,
        id: Uuid,
        old_sn: Uuid,
        new_sn: Uuid,
    ) -> SqlxResult<Result<(), SessionUpdateError>> {
        let expires_at = Utc::now().add(self.refresh_token_ttl);
        let result_code: i16 = sqlx::query_scalar("SELECT extend_session($1, $2, $3, $4)")
            .bind(id)
            .bind(old_sn)
            .bind(new_sn)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await?;

        match result_code {
            0 => Ok(Ok(())),
            1 => Ok(Err(SessionUpdateError::NotFound)),
            2 => Ok(Err(SessionUpdateError::InvalidSerialNumber)),
            3 => Ok(Err(SessionUpdateError::Expired)),
            _ => panic!("UNEXPECTED RETURN VALUE"), // Handle any other unexpected codes
        }
    }

    pub async fn remove_session(&self, id: Uuid) -> SqlxResult<()> {
        sqlx::query("DELETE FROM app_sessions WHERE id = $1;")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- PLAYLISTS ---

    pub async fn create_playlist(&self, title: &str, owner_id: i64) -> SqlxResult<i64> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO user_playlist (title, owner_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(title)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn delete_playlist(&self, playlist_id: i64) -> SqlxResult<()> {
        sqlx::query("DELETE FROM user_playlist WHERE id = $1")
            .bind(playlist_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_track_to_playlist(
        &self,
        playlist_id: i64,
        track_id: i64,
        platform: domain::models::db::user::TrackPlatform,
    ) -> SqlxResult<i64> {
        let id: i64 = sqlx::query_scalar("SELECT add_track_to_playlist($1, $2, $3)")
            .bind(playlist_id)
            .bind(track_id)
            .bind(platform)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn remove_track_from_playlist(&self, track_in_playlist_id: i64) -> SqlxResult<()> {
        sqlx::query("CALL remove_track_from_playlist($1)")
            .bind(track_in_playlist_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn change_track_position(
        &self,
        track_in_playlist_id: i64,
        new_position: i32,
    ) -> SqlxResult<()> {
        sqlx::query("CALL change_track_position($1, $2)")
            .bind(track_in_playlist_id)
            .bind(new_position)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
