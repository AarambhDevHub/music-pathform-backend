use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::{validate_email, Validate, ValidationError};
use regex::Regex;

use crate::models::{Duration, User};

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterUserDto {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    #[validate(custom = "validate_username")]
    pub username: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    pub password: String,

    #[
        validate(
            length(min = 1, message = "Confirm Password is required"),
            must_match(other = "password", message="passwords do not match")
        )
    ]
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !re.is_match(username) {
        return Err(ValidationError::new("Username can only contain letters, numbers, and underscores"));
    }

    Ok(())
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginUserDto {
    #[validate(custom = "validate_identifier")]
    pub identifier: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    pub password: String,
}

fn validate_identifier(identifier: &str) -> Result<(), ValidationError> {
    // You can define your own logic to differentiate between email and username
    if identifier.contains('@') {
       if !validate_email(identifier) {
            return Err(ValidationError::new("Invalid email format"));
       }
    } else if identifier.len() < 3 {
        return Err(ValidationError::new("Username must be at least 3 characters long"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterUserDto {
    pub id: String,
    pub username: String,
    pub email: String, 
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            username: user.username.to_owned(),
            email: user.email.to_owned(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user: FilterUserDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub user: FilterUserDto,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct NameUpdateDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct UserPasswordUpdateDto {
    #[validate(
        length(min = 1, message = "New password is required."),
        length(min = 6, message = "new password must be at least 6 characters")
    )]
    pub new_password: String,

    #[validate(
        length(min = 1, message = "New password confirm is required."),
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="new passwords do not match")
    )]
    pub new_password_confirm: String,

    #[validate(
        length(min = 1, message = "Old password is required."),
        length(min = 6, message = "Old password must be at least 6 characters")
    )]
    pub old_password: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
   pub track_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncompleteTrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub thumbnail_name: Option<String>,
    pub file_name: Option<String>,
    pub track_id: Option<uuid::Uuid>,
    pub total_chunks: i32,
    pub uploaded_chunks: i32,
    pub current_chunk: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteTrackInfoResponse {
    pub incomplete_track_info: Vec<IncompleteTrackInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterTrackDto {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration_minutes: f64, // Duration in minutes
    pub duration_seconds: f64, // Duration in seconds
    pub duration_played: f64, // Duration in seconds
    pub file_name: Option<String>,
    pub thumbnail_name: Option<String>,
    pub is_favorite: Option<bool>,
    pub played_at: Option<chrono::NaiveDateTime>,
    pub is_created_by_user: Option<bool>,
}

impl FilterTrackDto {
    pub fn filter_track(track: &TrackDto) -> Self {
        FilterTrackDto {
            id: track.id.clone(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            duration_minutes: convert_duration_to_minutes(&track.duration),
            duration_seconds: convert_duration_to_seconds(&track.duration),
            duration_played: convert_duration_to_seconds(&track.duration_played),
            file_name: track.file_name.clone(),
            thumbnail_name: track.thumbnail_name.clone(),      
            is_favorite: track.is_favorite.clone(),      
            played_at: track.played_at.clone(),
            is_created_by_user: track.is_created_by_user.clone()
        }
    }

    pub fn filter_tracks(track: &[TrackDto]) -> Vec<FilterTrackDto> {
        track.iter().map(FilterTrackDto::filter_track).collect()
    }
}

fn convert_duration_to_minutes(duration: &Duration) -> f64 {
    // Convert duration to total seconds
    let total_seconds = (duration.months * 30 * 24 * 60 * 60) as f64 // Approximation: 30 days in a month
        + (duration.days * 24 * 60 * 60) as f64
        + (duration.microseconds as f64 / 1_000_000.0); // Convert microseconds to seconds

    // Convert total seconds to minutes
    let total_minutes = total_seconds / 60.0;
    
    total_minutes
}

fn convert_duration_to_seconds(duration: &Duration) -> f64 {
    // Convert duration to total seconds
    let total_seconds = (duration.months * 30 * 24 * 60 * 60) as f64 // Approximation: 30 days in a month
        + (duration.days * 24 * 60 * 60) as f64
        + (duration.microseconds as f64 / 1_000_000.0); // Convert microseconds to seconds
   
    total_seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct  TrackResponseDto {
    pub tracks: Vec<FilterTrackDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveFavoritesDto {
    pub track_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackDto {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: Duration,
    pub file_name: Option<String>,
    pub upload_status: Option<String>,
    pub thumbnail_name: Option<String>,
    pub is_favorite: Option<bool>,
    pub duration_played: Duration,
    pub played_at: Option<chrono::NaiveDateTime>,
    pub is_created_by_user: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayListDto {
    pub id: uuid::Uuid,
    pub title: String,
    pub thumbnail_path: Option<String>,
    pub max_track_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayListResponse {
    pub playlists: Vec<PlayListDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTrackPlaylist {
    pub playlist_id: uuid::Uuid,
    pub track_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackMessageDto {
    pub track_id: uuid::Uuid,
    pub duration_played: i64, // Duration in seconds

}