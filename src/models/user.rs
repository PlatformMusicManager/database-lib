use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct UserTable {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionTable {
    id: Uuid,
    user_id: i64,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserPlaylist {
    id: i64,
    title: String,
    owner_id: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TrackInUserPlaylist {
    id: i64,
    title: String,
    track_id: i64,
    platform_id: TrackPlatfrom,
    position: i32,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "track_platform")]
#[sqlx(rename_all = "lowercase")]
enum TrackPlatfrom {
    Deezer,
    Soundloud,
}
