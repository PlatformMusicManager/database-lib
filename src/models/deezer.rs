// Types

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "album_input_deezer")]
pub struct AlbumInputDeezer {
    id: i64,
    title: String,
    img: Option<String>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "track_input_deezer")]
pub struct TrackInputDeezer {
    id: i64,
    title: String,
    duration: i32,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "author_input_deezer")]
pub struct AuthorInputDeezer {
    id: i64,
    title: String,
    img: Option<String>,
}

// Entity's

#[derive(Debug, sqlx::FromRow)]
pub struct TrackTableDeezer {
    id: i64,
    title: String,
    duration: i32,
    img: Option<String>,
    album_id: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AlbumTableDeezer {
    id: i64,
    title: String,
    img: Option<String>,
    author_id: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuthorTableDeezer {
    id: i64,
    title: String,
    img: Option<String>,
}
