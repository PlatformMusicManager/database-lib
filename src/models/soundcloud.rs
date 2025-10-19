// Types

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "author_input_soundcloud")]
pub struct AuthorInputSoundcloud {
    id: i64,
    title: String,
    img: Option<String>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "track_input_soundcloud")]
pub struct TrackInputSoundcloud {
    id: i64,
    title: String,
    duration: i32,
    img: Option<String>,
}

// Table

#[derive(Debug, sqlx::FromRow)]
pub struct TrackTableSoundcloud {
    id: i64,
    title: String,
    duration: i32,
    img: Option<String>,
    author_id: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuthorTableSoundcloud {
    id: i64,
    title: String,
    img: Option<String>,
}
