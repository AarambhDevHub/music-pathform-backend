use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgInterval, FromRow};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Duration {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            months: 0,          // Default value for months
            days: 0,            // Default value for days
            microseconds: 0,    // Default value for microseconds
        }
    }
}


impl From<PgInterval> for Duration {
    fn from(interval: PgInterval) -> Self {
        Duration {
            months: interval.months,
            days: interval.days,
            microseconds: interval.microseconds,
        }
    }
}

impl From<Option<PgInterval>> for Duration {
    fn from(option: Option<PgInterval>) -> Self {
        match option {
            Some(interval) => Duration::from(interval),
            None => Duration::default(), // Return default Duration if None
        }
    }
}

// User Model
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// Track Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Track {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: Duration,
    pub file_name: Option<String>,
    pub upload_status: Option<String>,
    pub thumbnail_name: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// AudioFile Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AudioFile {
    pub id: Uuid,
    pub track_id: Option<Uuid>,
    pub total_chunks: i32,
    pub uploaded_chunks: i32,
    pub current_chunk: i32,
    pub chunk_path: Option<String>,
    pub upload_status: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// Playlist Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub thumbnail_path: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// PlaylistTrack Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PlaylistTrack {
    pub playlist_id: Uuid,
    pub track_id: Uuid,
    pub track_order: i32,
}

// PlaybackHistory Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PlaybackHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub track_id: Uuid,
    pub played_at: NaiveDateTime,
    pub duration_played: Duration,
}

// UserFavorite Model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserFavorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub track_id: Uuid,
    pub created_at: NaiveDateTime,
}
